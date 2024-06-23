use cliproc::{cli, proc, stage::Memory, Arg, Cli, Help, Subcommand};

use crate::commands::helps::run;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::lang::LangMode;
use crate::core::lockfile::LockEntry;
use crate::core::target::{PluginError, Target};
use crate::core::variable::VariableTable;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::environment::{EnvVar, Environment, ORBIT_BLUEPRINT, ORBIT_BUILD_DIR};

use super::build::Build;
use super::plan::{self, Plan, BLUEPRINT_FILE};

#[derive(Debug, PartialEq)]
pub struct Run {
    target: Option<String>,
    args: Vec<String>,
    list: bool,
    target_dir: Option<String>,
    force: bool,
    all: bool,
    clean: bool,
    top: Option<Identifier>,
    bench: Option<Identifier>,
}

impl Subcommand<Context> for Run {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(run::HELP))?;
        Ok(Run {
            // Flags
            list: cli.check(Arg::flag("list"))?,
            force: cli.check(Arg::flag("force"))?,
            all: cli.check(Arg::flag("all"))?,
            clean: cli.check(Arg::flag("clean"))?,
            target_dir: cli.get(Arg::option("target-dir"))?,
            top: cli.get(Arg::option("top").value("unit"))?,
            target: cli.get(Arg::option("target"))?,
            bench: cli.get(Arg::option("bench").value("unit"))?,
            // Remaining args
            args: cli.remainder()?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // locate the plugin
        let target = match &self.target {
            // verify the plugin alias matches
            Some(name) => match c.get_config().get_plugins().get(name.as_str()) {
                Some(&t) => Some(t),
                None => return Err(PluginError::Missing(name.to_string()))?,
            },
            None => None,
        };

        // display plugin list and exit
        if self.list == true {
            match target {
                // display entire contents about the particular plugin
                Some(tar) => println!("{}", tar),
                // display quick overview of all plugins
                None => println!(
                    "{}",
                    Target::list_targets(
                        &mut c
                            .get_config()
                            .get_plugins()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Target>>()
                    )
                ),
            }
            return Ok(());
        }

        // check that user is in an IP directory
        c.goto_ip_path()?;

        // create the ip manifest
        let ip = Ip::load(c.get_ip_path().unwrap().clone(), true)?;

        // gather the catalog
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // determine the build directory (command-line arg overrides configuration setting)
        let default_build_dir = c.get_build_dir();
        let target_dir = match &self.target_dir {
            Some(dir) => dir,
            None => &default_build_dir,
        };

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if ip.can_use_lock() == true && self.force == false {
            let le: LockEntry = LockEntry::from((&ip, true));
            let lf = ip.get_lock();

            let env = Environment::new()
                // read config.toml for setting any env variables
                .from_config(c.get_config())?;
            let vtable = VariableTable::new().load_environment(&env)?;

            plan::download_missing_deps(
                vtable,
                &lf,
                &le,
                &catalog,
                &c.get_config().get_protocols(),
            )?;
            // recollect the downloaded items to update the catalog for installations
            catalog = catalog.downloads(c.get_downloads_path())?;

            plan::install_missing_deps(&lf, &le, &catalog)?;
            // recollect the installations to update the catalog for dependency graphing
            catalog = catalog.installations(c.get_cache_path())?;
        }

        self.run(ip, target_dir, target, catalog, &c.get_lang_mode())
    }
}

impl Run {
    fn run(
        &self,
        ip: Ip,
        target_dir: &str,
        target: Option<&Target>,
        catalog: Catalog,
        mode: &LangMode,
    ) -> Result<(), Fault> {
        // prepare for build
        let env = Environment::new()
            // read ip manifest for env variables
            .from_ip(&ip)?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE))
            .add(EnvVar::new().key(ORBIT_BUILD_DIR).value(target_dir));

        let env_path = ip.get_root().join(target_dir);

        // plan the target
        Plan::run(
            ip,
            target_dir,
            target,
            catalog,
            mode,
            self.clean,
            self.force,
            false,
            self.all,
            &self.bench,
            &self.top,
            &None,
        )?;

        env.initialize();

        // load from .env file from the correct build dir
        let envs: Environment = match Environment::new().from_env_file(&env_path) {
            Ok(r) => r,
            Err(e) => match self.force {
                false => return Err(AnyError(format!("failed to read .env file: {}", e)))?,
                true => Environment::new(),
            },
        };
        envs.initialize();

        // build the target
        Build::run(target, target_dir, &None, &self.args, false)
    }
}
