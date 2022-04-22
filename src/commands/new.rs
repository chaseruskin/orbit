use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::pkgid;
use crate::interface::arg::Arg;
use crate::core::context::Context;
use std::error::Error;

#[derive(Debug, PartialEq)]
pub struct New {
    ip: pkgid::PkgId,
    path: Option<std::path::PathBuf>,
}

impl Command for New {
    type Err = Box<dyn Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        // extra validation for a new IP spec to contain all fields (V.L.N)
        if let Err(e) = self.ip.fully_qualified() {
            return Err(Box::new(CliError::BadType(Arg::Positional(Positional::new("ip")), e.to_string())));
        }
        
        // an explicit environment variable takes precedence over config file data
        let root = std::path::PathBuf::from(match std::env::var("ORBIT_PATH") {
            Ok(v) => v,
            Err(_) => {
                let path = context.get_config().get("core").unwrap().get("path").unwrap().as_str().unwrap();
                std::env::set_var("ORBIT_PATH", &path);
                path.to_string()
            }
        });
        // :todo: verify the orbit path exists

        // only pass in necessary variables from context
        self.run(root, context.force)
    }
}

use crate::core::ip::IP;

impl New {
    fn run(&self, root: std::path::PathBuf, force: bool) -> Result<(), Box<dyn Error>> {
        // create ip stemming from ORBIT_PATH with default /VENDOR/LIBRARY/NAME
        let ip_path = if self.path.is_none() {
            root.join(self.ip.get_vendor().as_ref().unwrap())
                .join(self.ip.get_library().as_ref().unwrap())
                .join(self.ip.get_name())
        } else {
            root.join(self.path.as_ref().unwrap())
        };
        let ip = IP::new(ip_path, force)?.create_manifest(&self.ip)?;
        println!("info: new ip created at {}", ip.get_path().display());
        Ok(())
    }
}

impl FromCli for New {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(New {
            ip: cli.require_positional(Positional::new("ip"))?,
            path: cli.check_option(Optional::new("path"))?,
        });
        command
    }
}

const HELP: &str = "\
Create a new orbit ip package.

Usage:
    orbit new [options] <ip>

Args:
    <ip>                the V.L.N for the new package

Options:
    --path <path>       set the destination directory
    --template <key>    specify a template to copy

Use 'orbit help new' to read more about the command.
";