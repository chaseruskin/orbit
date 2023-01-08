use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Optional};
use clif::Error as CliError;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct Develop {
    flag: Option<String>,
}

impl FromCli for Develop {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Develop {
            flag: cli.check_option(Optional::new("flag"))?,
        });
        command
    }
}

impl Command<Context> for Develop {
    type Status = Result<(), Box<dyn std::error::Error>>;

    fn exec(&self, _: &Context) -> Self::Status {
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