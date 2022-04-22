use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;

#[derive(Debug, PartialEq)]
pub struct Search {
    ip: Option<PkgId>,
    cached: bool,
    developing: bool,
    available: bool,
}

impl Command for Search {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, _: &Context) -> Result<(), Self::Err> {
        Ok(self.run())
    }
}

impl Search {
    fn run(&self) -> () {
        // walk the ORBIT_PATH directory 

        // find all ip installed in cache

        // walk vendor directory to find all ip manifest available
    }
}

impl FromCli for Search {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Search {
            ip: cli.check_positional(Positional::new("ip"))?,
            cached: cli.check_flag(Flag::new("cache").switch('c'))?,
            developing: cli.check_flag(Flag::new("develop").switch('d'))?,
            available: cli.check_flag(Flag::new("available").switch('a'))?,
        });
        command
    }
}

const HELP: &str = "\
Browse and filter ip from the catalog.

Usage:
    orbit search [options] [<ip>]

Args:
    <ip>                a partially qualified pkgid to lookup ip

Options:
    --cache, -c         filter for ip installed to cache
    --develop, -d       filter for ip in-development
    --available, -a     filter for ip available from vendors

Use 'orbit help search' to learn more about the command.
";