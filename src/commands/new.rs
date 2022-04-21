use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional};
use crate::interface::errors::CliError;
use crate::core::pkgid;
use crate::interface::arg::Arg;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct New {
    ip: pkgid::PkgId,
}

impl Command for New {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        // extra validation for a new IP spec to contain all fields (V.L.N)
        if let Err(e) = self.ip.fully_qualified() {
            return Err(Box::new(CliError::BadType(Arg::Positional(Positional::new("ip")), e.to_string())));
        }

        let m = context.get_config().get("core").unwrap().get("path").unwrap();
        println!("orbit path: {}", m.as_str().unwrap());
        // :todo: only pass in necessary variables from context
        Ok(self.run())
    }
}

impl New {
    fn run(&self) -> () {
        println!("info: creating new ip {}", self.ip);
    }
}

impl FromCli for New {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(New {
            ip: cli.require_positional(Positional::new("ip"))?,
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
    --template <key>    specify a template to start from

Use 'orbit help new' to read more about the command.
";