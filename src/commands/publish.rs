use crate::core::channel::Channel;
use crate::core::context::Context;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use super::helps::publish::HELP;

#[derive(Debug, PartialEq)]
pub struct Publish {
    ready: bool,
    list: bool,
    no_install: bool,
}

impl Subcommand<Context> for Publish {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(HELP))?;
        Ok(Publish {
            list: cli.check(Arg::flag("list"))?,
            ready: cli.check(Arg::flag("ready").switch('y'))?,
            no_install: cli.check(Arg::flag("install"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // display channel list and exit
        if self.list == true {
            println!(
                "{}",
                Channel::list_channels(
                    &mut c
                        .get_config()
                        .get_channels()
                        .values()
                        .into_iter()
                        .collect::<Vec<&&Channel>>()
                )
            );
            return Ok(());
        }
        // by default, do not make any changes to the codebase/project (only print out diagnostics)
        todo!("verify the ip manifest is valid");
        // todo!("verify the lock file is generated and up to date");
        // todo!("verify there is no other ip with this name (and different uuid)");
        // todo!("verify the HDL graph can be generated without errors");
        // warn if there are no HDL units in the project
    }
}
