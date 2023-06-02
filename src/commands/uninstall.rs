use clif::cmd::{FromCli, Command};
use crate::core::v2::catalog::Catalog;
use crate::core::v2::ip::PartialIpSpec;
use clif::Cli;
use clif::arg::{Positional, Flag};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::OrbitResult;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Uninstall {
    ip: PartialIpSpec,
    full: bool,
    // @todo: add option to remove all versions (including store)
    // @todo: 
}

impl FromCli for Uninstall {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Uninstall {
            full: cli.check_flag(Flag::new("full"))?,
            ip: cli.require_positional(Positional::new("spec"))?,
        });
        command
    }
}

impl Command<Context> for Uninstall {

    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // collect the catalog from dev and installations
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .queue(c.get_queue_path())?;

        // check for ip in development or installation
        let status = match catalog.inner().get(&self.ip.get_name()) {
            Some(st) => st,
            None => {
                return Err(AnyError(format!("ip '{}' does not exist in the cache", self.ip)))?
            }
        };

        // grab the ip's manifest
        let target = match status.get_install(&self.ip.get_version()) {
                Some(t) => t,
                None => return Err(AnyError(format!("IP {} does not exist in the cache", self.ip)))?
        };

        // delete the project
        fs::remove_dir_all(target.get_root())?;

        // @TODO if force is off and the project is not found anywhere else, then ask
        // confirmation prompt

        self.run()
    }
}

impl Uninstall {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Remove an ip from the catalog.

Usage:
    orbit uninstall [options] <ip>

Args:
    <spec>      the name corresponding to the ip to delete
    --full      fully remove the ip and its dependencies

Use 'orbit help uninstall' to learn more about the command.
";