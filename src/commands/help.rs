use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional};
use crate::interface::errors::CliError;

#[derive(Debug, PartialEq)]
pub struct Help {
    topic: Option<String>,
}

impl Command for Help {
    fn exec(&self) -> () {
        self.run();
    }
}

impl Help {
    fn run(&self) -> () {
        println!("info: displaying help text");
    }
}

impl FromCli for Help {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Help {
            topic: cli.check_positional(Positional::new("topic"))?,
        });
        command
    }
}

const HELP: &str = "\
Read in-depth documentation around Orbit topics.

Usage:
    orbit help [<topic>]

Args:
    <topic>         a listed topic or any orbit subcommand

Topics:
    manifest        learn about .cfg files
    cache           learn about orbit's caching system

Use 'orbit help --list' to see all available topics.
";