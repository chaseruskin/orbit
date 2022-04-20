use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional};
use crate::interface::errors::CliError;
use crate::core::pkgid;

#[derive(Debug, PartialEq)]
pub struct New {
    ip: pkgid::PkgId,
}

impl Command for New {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self) -> Result<(), Self::Err> {
        self.ip.fully_qualified()?;
        Ok(self.run())
    }
}

impl New {
    fn run(&self) -> () {
        println!("info: creating new ip");
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
    orbit new <ip>

Args:
    <ip>                the V.L.N for the new package

Options:
    --template <key>    specify a template to start from

Use 'orbit help new' read more about the command.
";