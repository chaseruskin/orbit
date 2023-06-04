use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Flag, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::core::version::Version;
use crate::OrbitResult;

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
    no_install: bool,
}

impl FromCli for Launch {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Launch {
            ready: cli.check_flag(Flag::new("ready"))?,
            next: cli.check_option(Optional::new("next").value("version"))?,
            message: cli.check_option(Optional::new("message").switch('m'))?,
            no_install: cli.check_flag(Flag::new("no-install"))?,
        });
        command
    }
}

impl Command<Context> for Launch {
    type Status = OrbitResult;

    fn exec(&self, _c: &Context) -> Self::Status {
        todo!();
    }
}

const HELP: &str = "\
Releases (tags) the current ip's latest commit as the next version.

Usage:
    orbit launch [options]

Options:
    --ready                 proceed with the launch process
    --next <version>        semver version or 'major', 'minor', or 'patch'
    --message, -m <message> message to apply to the commit when using '--next'
    --no-install            skip installing newly launched version

Use 'orbit help launch' to learn more about the command.
";