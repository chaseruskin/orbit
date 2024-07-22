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

use std::collections::HashMap;

use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::catalog::Catalog;
use crate::core::channel::Channel;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::Language;
use crate::error::{Error, Hint, LastError};
use crate::util::anyerror::Fault;

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

        let ip_spec = local_ip.get_man().get_ip().into_ip_spec();

        println!("info: {}", "finding channels to publish to ...");
        let mut channels = HashMap::new();
        // verify default channel is valid
        match c.get_config().get_default_channel() {
            Some(name) => match c.get_config().get_channels().get(name) {
                Some(&chan) => {
                    println!("info: using default channel {:?}", name);
                    channels.insert(name, chan)
                }
                None => return Err(Box::new(Error::DefChanNotFound(name.clone())))?,
            },
            // no default channel selected
            None => None,
        };

        // check the channel(s) we wish to publish to
        let ip_channels = local_ip.get_man().get_ip().get_channels();
        for name in ip_channels {
            match c.get_config().get_channels().get(name) {
                Some(&chan) => {
                    println!("info: using specified channel {:?}", name);
                    channels.insert(name, chan)
                }
                None => return Err(Box::new(Error::ChanNotFound(name.clone())))?,
            };
        }

        // make sure a channel is configured
        if channels.is_empty() == true {
            return Err(Box::new(Error::NoChanDefined))?;
        }

        // verify the version of the ip does not already exist at the available level
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&channels)?;

        if let Some(found) = catalog.inner().get(local_ip.get_man().get_ip().get_name()) {
            match found.get_available(&crate::core::version::AnyVersion::Specific(
                local_ip
                    .get_man()
                    .get_ip()
                    .get_version()
                    .to_partial_version(),
            )) {
                Some(_) => {
                    return Err(Box::new(Error::PublishAlreadyExists(ip_spec)))?;
                }
                None => {}
            }
        }

        // verify the lock file is generated and up to date
        println!("info: {}", "verifying lockfile is up to date ...");
        if local_ip.can_use_lock() == false {
            return Err(Box::new(Error::PublishMissingLockfile(Hint::MakeLock)));
        }

        // verify the ip has a source
        println!(
            "info: {}",
            "verifying ip manifest's source field is defined ..."
        );
        if local_ip.get_man().get_ip().get_source().is_none() {
            return Err(Box::new(Error::PublishMissingSource));
        }

        // verify the graph build with no errors
        println!("info: {}", "verifying hardware graph construction ...");
        if let Err(e) = Self::check_graph_builds_okay(&local_ip, &catalog) {
            return Err(Box::new(Error::PublishHdlGraphFailed(LastError(
                e.to_string(),
            ))))?;
        }

        // by default, do not make any changes to the codebase/project (only print out diagnostics)
        // todo!("verify the lock file is generated and up to date");
        // todo!("verify there is no other ip with this name (and different uuid)");
        // todo!("verify the HDL graph can be generated without errors");
        // warn if there are no HDL units in the project
        match self.ready {
            true => self.publish(&local_ip, channels),
            false => Err(Box::new(Error::PublishDryRunDone(
                ip_spec,
                Hint::PublishWithReady,
            )))?,
        }
    }
}

impl Publish {
    fn check_graph_builds_okay(local_ip: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        // use all language settings
        let lang = Language::default();
        let ip_graph = algo::compute_final_ip_graph(&local_ip, &catalog, &lang)?;
        let files = algo::build_ip_file_list(&ip_graph, &local_ip, &lang);
        let _global_graph = Plan::build_full_graph(&files)?;
        Ok(())
    }

    fn publish(self, local_ip: &Ip, channels: HashMap<&String, &Channel>) -> Result<(), Fault> {
        todo!()
    }
}
