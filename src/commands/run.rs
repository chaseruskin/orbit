use cliproc::{cli, proc, stage::Memory, Arg, Cli, Help, Subcommand};

use crate::commands::helps::run;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct Run {
    target: Option<String>,
    args: Vec<String>,
}

impl Subcommand<Context> for Run {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(run::HELP))?;
        Ok(Run {
            // Flags
            target: cli.get(Arg::option("target"))?,
            // Remaining args
            args: cli.remainder()?,
        })
    }

    fn execute(self, _c: &Context) -> proc::Result {
        todo!();
        // Ok(())
    }
}
