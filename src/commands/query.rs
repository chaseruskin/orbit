use crate::Command;
use crate::FromCli;
use crate::core::pkgid::PkgId;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Query {
    ip: PkgId,
    tags: bool,
}

impl FromCli for Query {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Query {
            tags: cli.check_flag(Flag::new("tags"))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        });
        command
    }
}

impl Command for Query {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // collect all ip in the user's universe
        self.run()
    }
}

impl Query {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Probe information about an ip

Usage:
    orbit query [options] <ip>

Args:
    <ip>               the pkgid to request data about

Options:
    --tags
    --install, -i <version>
    --available, -a <version>
    --develop, -d,
    --units

Use 'orbit help query' to learn more about the command.
";