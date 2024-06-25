use cliproc::{cli, proc, stage::Memory, Arg, Cli, Help, Subcommand};

use crate::commands::helps::run;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::lang::LangMode;
use crate::core::target::Process;
use crate::core::target::Target;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::environment::{EnvVar, Environment, ORBIT_BLUEPRINT, ORBIT_BUILD_DIR};

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
    verbose: bool,
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
            verbose: cli.check(Arg::flag("verbose"))?,
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
        let target = c.select_target(&self.target, self.list == false)?;

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
                            .get_targets()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Target>>()
                    )
                ),
            }
            return Ok(());
        }

        let target = target.unwrap();

        // check that user is in an IP directory
        c.jump_to_working_ip()?;

        // create the ip manifest
        let ip = Ip::load(c.get_ip_path().unwrap().clone(), true)?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // determine the build directory (command-line arg overrides configuration setting)
        let default_build_dir = c.get_target_dir();
        let target_dir = match &self.target_dir {
            Some(dir) => dir,
            None => &default_build_dir,
        };

        // gather the catalog and resolve any missing dependencies
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;
        let catalog = plan::resolve_missing_deps(c, &ip, catalog, self.force)?;

        self.run(&ip, target_dir, target, catalog, &c.get_lang_mode())
    }
}

impl Run {
    fn run(
        &self,
        ip: &Ip,
        target_dir: &str,
        target: &Target,
        catalog: Catalog,
        mode: &LangMode,
    ) -> Result<(), Fault> {
        // plan the target
        Plan::run(
            &ip,
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

        // prepare for build
        Environment::new()
            // read ip manifest for env variables
            .from_ip(&ip)?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE))
            .add(EnvVar::new().key(ORBIT_BUILD_DIR).value(target_dir))
            .initialize();

        let env_path = ip.get_root().join(target_dir);

        // load from .env file from the correct build dir
        let envs: Environment = match Environment::new().from_env_file(&env_path) {
            Ok(r) => r,
            Err(e) => match self.force {
                false => return Err(AnyError(format!("failed to read .env file: {}", e)))?,
                true => Environment::new(),
            },
        };
        envs.initialize();

        let output_path = ip.get_root().join(target_dir).join(&target.get_name());

        // run the command from the output path
        target.execute(&None, &self.args, self.verbose, &output_path)
    }
}
