use crate::core::context::Context;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use super::helps::launch::HELP;

#[derive(Debug, PartialEq)]
pub struct Launch {
    ready: bool,
    no_install: bool,
}

impl Subcommand<Context> for Launch {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(HELP))?;
        Ok(Launch {
            ready: cli.check(Arg::flag("ready"))?,
            no_install: cli.check(Arg::flag("install"))?,
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
