use clif::cmd::{FromCli, Command};
use crate::core::v2::catalog::Catalog;
use crate::core::pkgid::PkgPart;
use crate::core::version::AnyVersion;
use clif::Cli;
use clif::arg::{Positional, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::OrbitResult;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Uninstall {
    ip: PkgPart,
    version: AnyVersion
    // @TODO add option to remove all versions (including store)
}

impl FromCli for Uninstall {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Uninstall {
            ip: cli.require_positional(Positional::new("ip"))?,
            version: cli.check_option(Optional::new("ver").switch('v').value("version"))?.unwrap_or(AnyVersion::Latest),
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

        // @deprec: find the target IP
        // let ids = catalog.inner().keys().map(|f| { f }).collect();
        // let target = crate::core::ip::find_ip(&self.ip, ids)?;

        // check for ip in development or installation
        let status = match catalog.inner().get(&self.ip) {
            Some(st) => st,
            None => {
                return Err(AnyError(format!("ip '{}' does not exist in the cache", self.ip)))?
            }
        };

        // grab the ip's manifest
        let target = match status.get_install(&self.version) {
                Some(t) => t,
                None => return Err(AnyError(format!("ip '{}' is not installed to the cache under version '{}'", self.ip, self.version)))?
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
    <ip>                    the name corresponding to the ip to delete
    --ver, -v <version>     the version of the IP to remove

Use 'orbit help uninstall' to learn more about the command.
";