//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use crate::core::catalog::Catalog;
use crate::core::channel::Channel;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::error::{Error, Hint};

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

        // verify running from an ip directory and enter ip's root directory
        c.jump_to_working_ip()?;

        let local_ip = Ip::load(c.get_ip_path().unwrap().to_path_buf(), true)?;

        // verify the version of the ip does not already exist at the available level
        let _catalog = Catalog::new().available(c.get_config().get_channels())?;

        // verify the lock file is generated and up to date
        if local_ip.can_use_lock() == false {
            return Err(Box::new(Error::PublishMissingLockfile(Hint::MakeLock)));
        }

        // verify the ip has a source
        if local_ip.get_man().get_ip().get_source().is_none() {
            return Err(Box::new(Error::PublishMissingSource));
        }

        // by default, do not make any changes to the codebase/project (only print out diagnostics)
        todo!("verify the ip manifest is valid");
        // todo!("verify the lock file is generated and up to date");
        // todo!("verify there is no other ip with this name (and different uuid)");
        // todo!("verify the HDL graph can be generated without errors");
        // warn if there are no HDL units in the project
    }
}
