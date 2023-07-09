use crate::core::context::Context;
use crate::core::version::Version;
use crate::OrbitResult;
use clif::arg::{Flag, Optional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;

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
    install: bool,
}

impl FromCli for Launch {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Launch {
            ready: cli.check_flag(Flag::new("ready"))?,
            install: cli.check_flag(Flag::new("install"))?,
            next: cli.check_option(Optional::new("next").value("version"))?,
        });
        command
    }
}

impl Command<Context> for Launch {
    type Status = OrbitResult;

    fn exec(&self, _c: &Context) -> Self::Status {
        // by default, do not make any changes to the codebase/project (only print out diagnostics)
        todo!("verify the ip manifest is valid");
        // todo!("verify the lock file is generated and up to date");
        // todo!("verify there is no other ip with this name (and different uuid)");
        // todo!("verify the HDL graph can be generated without errors");
        // warn if there are no HDL units in the project
    }
}

const HELP: &str = "\
Run a series of checks to verify the ip is ready to be released.

Usage:
    orbit launch [options]

Options:
    --ready                 proceed with the launch process
    --next <version>        semver version or 'major', 'minor', or 'patch'
    --install               install the newly launched version

Use 'orbit help launch' to learn more about the command.
";
