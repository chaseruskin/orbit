use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Positional};
use crate::interface::errors::CliError;

#[derive(Debug, PartialEq)]
pub struct Orbit {
    help: bool,
    upgrade: bool,
    version: bool,
    command: Option<OrbitSubcommand>,
}

impl Command for Orbit {
    fn exec(&self) -> () {
        self.run();
    }
}

impl Orbit {
    fn run(&self) -> () {
        // prioritize version information
        if self.version {
            println!("0.1.0");
        // run the specified command
        } else if let Some(c) = &self.command {
            c.exec();
        // if no command is given then print default help
        } else {
            println!("{}", HELP);
        }
    }
}

impl FromCli for Orbit {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let orbit = Ok(Orbit {
            help: cli.check_flag(Flag::new("help").switch('h'))?,
            version: cli.check_flag(Flag::new("version"))?,
            upgrade: cli.check_flag(Flag::new("upgrade"))?,
            command: cli.check_command(Positional::new("command"))?,
        });
        orbit
    }
}

use crate::commands::help::Help;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
}

impl FromCli for OrbitSubcommand {
    fn from_cli<'c>(cli: &'c mut Cli<'_>) -> Result<Self, CliError<'c>> { 
        match cli.match_command(&["help"])?.as_ref() {
            "help" => Ok(OrbitSubcommand::Help(Help::from_cli(cli)?)),
            _ => panic!("an unimplemented command was passed through!")
        }
    }
}

impl Command for OrbitSubcommand {
    fn exec(&self) {
        match self {
            OrbitSubcommand::Help(c) => c.exec(),
        }
    }
}

const HELP: &str = "\
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
";