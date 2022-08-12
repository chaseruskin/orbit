use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct Develop {
    flag: Option<String>,
}

impl FromCli for Develop {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Develop {
            flag: cli.check_option(Optional::new("flag"))?,
        });
        command
    }
}

impl Command for Develop {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        self.run()
    }
}

impl Develop {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Bring an ip to the development state for editing.

Usage:
    orbit develop [options]

Options:
    --ip <pkgid>           
    --git <url>
    --variant, -v <version>
    --path <path>
    --to <path>

Use 'orbit help develop' to learn more about the command.
";