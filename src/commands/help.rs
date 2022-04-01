use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;

#[derive(Debug, PartialEq)]
pub struct Help {
    man: bool,
    topic: Option<String>,
}

impl Command for Help {
    fn exec(&self) -> () {
        self.run();
    }
}

impl Help {
    fn run(&self) -> () {
        println!("running command: help"); 
    }
}

impl FromCli for Help {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Help {
            man: cli.check_flag(Flag::new("man"))?,
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
    topic1          information about topic1
    topic2          information about topic2

Use 'orbit help <topic>' for more information about a command.
";