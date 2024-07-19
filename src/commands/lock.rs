use super::plan::{self, Plan};
use crate::commands::helps::lock;
use crate::core::algo;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::Language;
use crate::core::lockfile::LockEntry;
use crate::core::swap::StrSwapTable;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

pub const BLUEPRINT_FILE: &str = "blueprint.tsv";
pub const BLUEPRINT_DELIMITER: &str = "\t";

#[derive(Debug, PartialEq)]
pub struct Lock {
    force: bool,
}

impl Subcommand<Context> for Lock {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(lock::HELP))?;
        let command = Ok(Lock {
            // flags
            force: cli.check(Arg::flag("force"))?,
        });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        // check that user is in an IP directory
        c.jump_to_working_ip()?;

        // store the working ip struct
        let working_ip = Ip::load(c.get_ip_path().unwrap().clone(), true)?;

        // assemble the catalog
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if working_ip.can_use_lock() == true && self.force == false {
            let le: LockEntry = LockEntry::from((&working_ip, true));
            let lf = working_ip.get_lock();

            let env = Environment::new()
                // read config.toml for setting any env variables
                .from_config(c.get_config())?;
            let vtable = StrSwapTable::new().load_environment(&env)?;

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

        Self::run(&working_ip, &catalog, &c.get_languages(), self.force)
    }
}

impl Lock {
    /// Performs the backend logic for creating a blueprint file (planning a design).
    pub fn run(
        working_ip: &Ip,
        catalog: &Catalog,
        lang: &Language,
        force: bool,
    ) -> Result<(), Fault> {
        // build entire ip graph and resolve with dynamic symbol transformation
        let ip_graph = match algo::compute_final_ip_graph(&working_ip, &catalog, lang) {
            Ok(g) => g,
            Err(e) => return Err(e)?,
        };

        // only write lockfile and exit if flag is raised
        Plan::write_lockfile(&working_ip, &ip_graph, force)?;
        Ok(())
    }
}
