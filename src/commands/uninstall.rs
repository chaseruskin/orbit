use clif::cmd::{FromCli, Command};
use crate::core::catalog::Catalog;
use crate::core::pkgid::PkgId;
use crate::core::version::AnyVersion;
use clif::Cli;
use clif::arg::{Positional, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::OrbitResult;

#[derive(Debug, PartialEq)]
pub struct Uninstall {
    ip: PkgId,
    version: AnyVersion
    // @TODO add option to remove all versions (including store)
}

impl FromCli for Uninstall {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Uninstall {
            ip: cli.require_positional(Positional::new("ip"))?,
            version: cli.check_option(Optional::new("variant").switch('v').value("version"))?.unwrap_or(AnyVersion::Dev),
        });
        command
    }
}

impl Command<Context> for Uninstall {

    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // collect the catalog from dev and installations
        let catalog = Catalog::new()
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(c.get_vendors())?;
        // find the target IP
        let ids = catalog.inner().keys().map(|f| { f }).collect();
        let target = crate::core::ip::find_ip(&self.ip, ids)?;

        // check for ip in development or installation (safe to unwrap because we took pkgid from keys)
        let status = catalog.inner().get(&target).unwrap();

        // grab the ip's manifest
        let manifest = match &self.version {
            AnyVersion::Dev => {
                match status.get_dev() {
                    Some(m) => m,
                    None => return Err(AnyError(format!("ip '{}' is not found in the DEV_PATH", self.ip)))?
                }
            },
            _ => match status.get_install(&self.version) {
                Some(m) => m,
                None => return Err(AnyError(format!("ip '{}' is not installed to the cache under version '{}'", self.ip, self.version)))?
            }
        };

        // delete the project
        manifest.remove()?;

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
Remove an ip from the catalog

Usage:
    orbit uninstall [options] <ip>

Args:
    <ip>                    the pkgid corresponding to the ip to delete
    --variant, -v <version> the version of the pkgid to remove

Use 'orbit help uninstall' to learn more about the command.
";