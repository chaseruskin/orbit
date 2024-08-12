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
use std::path::PathBuf;

use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::catalog::{Catalog, PointerSlot};
use crate::core::channel::Channel;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::lang::Language;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::error::{Error, Hint, LastError};
use crate::util::anyerror::Fault;
use crate::util::environment::{EnvVar, Environment, ORBIT_CHAN_INDEX};
use crate::util::filesystem;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use super::helps::publish::HELP;

#[derive(Debug, PartialEq)]
pub struct Publish {
    ready: bool,
    list: bool,
}

impl Subcommand<Context> for Publish {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(HELP))?;
        Ok(Publish {
            list: cli.check(Arg::flag("list"))?,
            ready: cli.check(Arg::flag("ready").switch('y'))?,
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

        // initialize environment
        let env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&local_ip)?;

        let ip_spec = local_ip.get_man().get_ip().into_ip_spec();

        println!("info: {}", "finding channels to publish to ...");
        let mut channels = HashMap::new();

        // check the channel(s) we wish to publish to
        if let Some(ip_channels) = local_ip.get_man().get_ip().get_channels() {
            for name in ip_channels {
                match c.get_config().get_channels().get(name) {
                    Some(&chan) => {
                        println!("info: using specified channel {:?}", name);
                        channels.insert(name, chan)
                    }
                    None => return Err(Box::new(Error::ChanNotFound(name.clone())))?,
                };
            }
        // try the default if channels are not defined in manifest
        } else {
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
        }

        // make sure a channel is configured
        if channels.is_empty() == true {
            return Err(Box::new(Error::NoChanDefined))?;
        }

        // run the synchronizations for each channel being used
        for (_name, chan) in &channels {
            chan.run_sync(&env)?;
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

        if let Err(e) = Self::run_ip_checkpoints(&local_ip, &catalog) {
            return Err(Box::new(Error::PublishFailedCheckpoint(LastError(
                e.to_string(),
            ))));
        }

        // TODO: warn if there are no HDL units in the project
        match self.ready {
            true => self.publish_all(&local_ip, channels, env),
            false => Err(Box::new(Error::PublishDryRunDone(
                ip_spec,
                Hint::PublishWithReady,
            )))?,
        }
    }
}

impl Publish {
    pub fn run_ip_checkpoints(local_ip: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        // verify the lock file is generated and up to date
        println!("info: {}", "verifying lockfile is up to date ...");
        if local_ip.can_use_lock() == false {
            return Err(Box::new(Error::PublishMissingLockfile(Hint::MakeLock)));
        }

        // verify the ip has zero relative dependencies
        println!("info: {}", "verifying all dependencies are stable ...");
        if let Some(dep) = local_ip.get_lock().inner().iter().find(|f| f.is_relative()) {
            return Err(Box::new(Error::PublishRelativeDepExists(
                dep.get_name().clone(),
            )));
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

        Ok(())
    }

    pub fn check_graph_builds_okay(local_ip: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        // use all language settings
        let lang = Language::default();
        let ip_graph = algo::compute_final_ip_graph(&local_ip, &catalog, &lang)?;
        let files = algo::build_ip_file_list(&ip_graph, &local_ip, &lang);
        let _global_graph = Plan::build_full_graph(&files)?;
        Ok(())
    }

    fn publish_all(
        &self,
        local_ip: &Ip,
        channels: HashMap<&String, &Channel>,
        mut env: Environment,
    ) -> Result<(), Fault> {
        // publish to each channel
        for (name, chan) in &channels {
            println!("info: publishing to {:?} channel ...", name);
            // update the index path
            let index_dir = Self::create_pointer_directory(&local_ip);
            let index_path = filesystem::into_std_str(chan.get_root().join(index_dir));
            env = env.overwrite(EnvVar::with(ORBIT_CHAN_INDEX, index_path.as_str()));
            // publish to this channel
            match self.publish(local_ip, chan, &env) {
                Ok(_) => (),
                Err(e) => {
                    self.rollback_changes(local_ip, chan)?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn publish(&self, local_ip: &Ip, channel: &Channel, env: &Environment) -> Result<(), Fault> {
        // run the pre-publish command sequence, if exist
        channel.run_pre(&env)?;
        // copy the ip's manifest to the location in the channel
        self.copy_to_channel(local_ip, channel)?;
        // run the post-publish command sequence, if exist
        channel.run_post(&env)?;
        Ok(())
    }

    /// Creates the path where an ip will place its pointer contents.
    ///
    /// The directory is something like this: name[0]/name-version-uuid
    fn create_pointer_directory(ip: &Ip) -> PathBuf {
        let name = ip.get_man().get_ip().get_name();
        let version = ip.get_man().get_ip().get_version();
        let uuid = ip.get_uuid();
        PathBuf::new()
            .join(String::from(name.as_ref().chars().next().unwrap()))
            .join(PointerSlot::new(name, version, uuid).as_ref())
    }

    /// Writes the ip's manifest to the channel.
    fn copy_to_channel(&self, local_ip: &Ip, channel: &Channel) -> Result<(), Fault> {
        let output_dir = Self::create_pointer_directory(&local_ip);
        let output_path = channel.get_root().join(output_dir);
        // create any mising directories
        std::fs::create_dir_all(&output_path)?;
        // copy the (raw) manifest there (in formatted string)
        std::fs::write(
            output_path.join(IP_MANIFEST_FILE),
            local_ip.get_man().to_string(),
        )?;
        // std::fs::copy(
        //     local_ip.get_root().join(IP_MANIFEST_FILE),
        //     output_path.join(IP_MANIFEST_FILE),
        // )?;
        Ok(())
    }

    /// Rollback the progress made during publish if an error has occurred.
    ///
    /// This function should be called before returning the final error from the publish
    /// operation to allow users to try again from a known state.
    fn rollback_changes(&self, local_ip: &Ip, channel: &Channel) -> Result<(), Fault> {
        let index_dir = Self::create_pointer_directory(&local_ip);
        let index_path = channel.get_root().join(index_dir);

        if index_path.exists() && index_path.is_dir() {
            std::fs::remove_dir_all(index_path)?;
        }
        // check if we should remove the first-layer directory
        let first_dir = PathBuf::from(String::from(
            local_ip
                .get_man()
                .get_ip()
                .get_name()
                .as_ref()
                .chars()
                .next()
                .unwrap(),
        ));
        let output_path = channel.get_root().join(first_dir);
        if output_path.exists() && output_path.is_dir() {
            match std::fs::read_dir(&output_path)?.count() {
                0 => std::fs::remove_dir(&output_path)?,
                _ => (),
            }
        }
        Ok(())
    }
}
