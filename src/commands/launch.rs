use crate::core::context::Context;
use crate::core::version::Version;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

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

impl Subcommand<Context> for Launch {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(HELP))?;
        Ok(Launch {
            ready: cli.check(Arg::flag("ready"))?,
            install: cli.check(Arg::flag("install"))?,
            next: cli.get(Arg::option("next").value("version"))?,
        })
    }

    fn execute(self, _c: &Context) -> proc::Result {
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
