use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::version::Version;

#[derive(Debug, PartialEq)]
enum VersionField {
    Major,
    Minor,
    Patch,
    Version(Version),
}

impl std::str::FromStr for VersionField {
    type Err = crate::core::version::VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            _ => Ok(Self::Version(Version::from_str(s)?)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Launch {
    next: Option<VersionField>,
    ready: bool,
    message: Option<String>,
}

impl FromCli for Launch {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Launch {
            ready: cli.check_flag(Flag::new("ready"))?,
            next: cli.check_option(Optional::new("next").value("version"))?,
            message: cli.check_option(Optional::new("message"))?,
        });
        command
    }
}

impl Command for Launch {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, _: &Context) -> Result<(), Self::Err> {
        // make sure it is run from an ip directory
        self.run()
    }
}

impl Launch {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        todo!()
    }
}

const HELP: &str = "\
Releases (tags) the current ip's latest commit as the next version.

Usage:
    orbit launch [options]

Options:
    --ready              proceed with the launch process
    --next <version>     semver version or 'major', 'minor', or 'patch'
    --message <message>  message to apply to the commit when using '--next'

Use 'orbit help launch' to learn more about the command.
";