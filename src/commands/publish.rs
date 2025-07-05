//
//  Copyright (C) 2022-2025  Chase Ruskin
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
use std::fs;
use std::path::PathBuf;

use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::catalog::{Catalog, PointerSlot};
use crate::core::channel::Channel;
use crate::core::context::Context;
use crate::core::manifest::JsonMeta;
use crate::core::manifest::{PROJECT_JSON_FILE, PROJECT_MANIFEST_FILE};
use crate::core::project::Project;
use crate::core::project_archive::ProjectArchive;
use crate::error::{Error, Hint, LastError};
use crate::util::anyerror::Fault;
use crate::util::environment::{
    EnvVar, Environment, ORBIT_CHANNEL_DIR, ORBIT_CHANNEL_NAME, ORBIT_CHANNEL_PROJECT_DIR,
};
use crate::util::filesystem;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use super::helps::publish::HELP;
use super::install::Install;
use super::remove::Remove;

#[derive(Debug, PartialEq)]
pub struct Publish {
    ready: bool,
    no_install: bool,
    list: bool,
    all_pub: bool,
    channels: Option<Vec<String>>,
}

impl Subcommand<Context> for Publish {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(HELP))?;
        Ok(Publish {
            list: cli.check(Arg::flag("list").switch('l'))?,
            no_install: cli.check(Arg::flag("no-install"))?,
            all_pub: cli.check(Arg::flag("all-public"))?,
            ready: cli.check(Arg::flag("ready").switch('y'))?,
            channels: cli.get_all(Arg::option("channel").switch('c'))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // display channel list and exit
        if self.list == true {
            // try to list the channels provided on command-line
            if let Some(channels) = &self.channels {
                // verify all requested channels exist
                for ch in channels {
                    if c.get_config()
                        .get_channels()
                        .iter()
                        .find(|(&n, _)| ch == n)
                        .is_none()
                    {
                        return Err(Box::new(Error::ChanNotFound(ch.clone())))?;
                    }
                }
                // print all requested channels
                c.get_config()
                    .get_channels()
                    .iter()
                    .filter(|(&n, _)| channels.iter().find(|&z| z == n).is_some())
                    .for_each(|(_, &d)| {
                        println!("{}", d.to_string());
                    });
            } else {
                print!(
                    "{}",
                    Channel::list_channels(
                        &mut c
                            .get_config()
                            .get_channels()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Channel>>(),
                        c.get_config().get_default_channels(),
                    )
                );
            }
            return Ok(());
        }

        // verify running from an ip directory and enter ip's root directory
        c.jump_to_working_project()?;

        let local_ip = Project::load(c.get_project_path().unwrap().to_path_buf(), true, false)?;

        // initialize environment
        let env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_project(&local_ip)?;

        let ip_spec = local_ip.get_man().get_project().into_project_id_spec();

        crate::info!("finding channels to publish to ...");
        let mut channels = HashMap::new();

        // determine from which list of channels should we try to publish to

        // first try the command-line
        let mut channel_candidates = self.channels.as_ref();

        // next try the list from the ip's manifest
        if channel_candidates.is_none() {
            channel_candidates = local_ip.get_man().get_project().get_channels().as_ref();
        }

        // check the channel(s) we wish to publish to (not defaults)
        if let Some(ip_channels) = channel_candidates {
            for name in ip_channels {
                match c.get_config().get_channels().get(name) {
                    Some(&chan) => {
                        crate::info!("using specified channel {:?}", name);
                        channels.insert(name, chan)
                    }
                    None => return Err(Box::new(Error::ChanNotFound(name.clone())))?,
                };
            }
        // try the default if channels are not defined in manifest
        } else {
            // verify default channel is valid
            if let Some(ip_channels) = c.get_config().get_default_channels() {
                for name in ip_channels {
                    match c.get_config().get_channels().get(name) {
                        Some(&chan) => {
                            crate::info!("using default channel {:?}", name);
                            channels.insert(name, chan)
                        }
                        // no default channel found in known channels
                        None => return Err(Box::new(Error::DefChanNotFound(name.clone())))?,
                    };
                }
            }
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

        if let Some(found) = catalog.translate_name(&ip_spec.to_pkg_name())? {
            match found.get_available(&crate::core::version::AnyVersion::Specific(
                local_ip
                    .get_man()
                    .get_project()
                    .get_version()
                    .to_partial_version(),
            )) {
                Some(_) => {
                    return Err(Box::new(Error::PublishAlreadyExists(ip_spec)))?;
                }
                None => {}
            }
        }

        if local_ip.has_public_list() == true && self.all_pub == true {
            return Err(Box::new(Error::IpAllPublicNotNow(LastError(
                Error::VisNoAllPubEntryExists.to_string(),
            ))));
        }

        if let Err(e) = Self::run_ip_checkpoints(
            &local_ip,
            &catalog,
            c.are_units_private_by_default(),
            self.all_pub,
        ) {
            return Err(Box::new(Error::PublishFailedCheckpoint(LastError(
                e.to_string(),
            ))));
        }

        // verify the package is available to be downloaded
        crate::info!("verifying coherency with project's source  ...");
        let remove = self.ready == false || self.no_install == true;
        let changes = match Self::test_download_and_install(&local_ip, &c, remove, true) {
            Ok(c) => c,
            Err(e) => {
                return Err(Box::new(Error::PublishFailedCheckpoint(LastError(
                    e.to_string(),
                ))));
            }
        };

        // TODO: warn if there are no HDL units in the project
        match self.ready {
            true => self.publish_all(&local_ip, channels, env, &changes),
            false => Err(Box::new(Error::PublishDryRunDone(
                ip_spec,
                Hint::PublishWithReady,
            )))?,
        }
    }
}

impl Publish {
    pub fn run_ip_checkpoints(
        local_ip: &Project,
        catalog: &Catalog,
        priv_by_def: bool,
        all_pub: bool,
    ) -> Result<(), Fault> {
        // verify the lock file is generated and up to date
        crate::info!("verifying lockfile is up to date ...");
        if local_ip.can_use_lock(&catalog) == false {
            return Err(Box::new(Error::PublishMissingLockfile(Hint::MakeLock)));
        }

        // verify the ip has zero relative dependencies
        crate::info!("verifying all dependencies are stable ...");
        if let Some(dep) = local_ip.get_lock().inner().iter().find(|f| f.is_relative()) {
            return Err(Box::new(Error::PublishRelativeDepExists(
                dep.get_name().clone(),
            )));
        }

        // verify the ip has a source
        crate::info!("verifying project manifest's source field is defined ...");
        if local_ip.get_man().get_project().get_source().is_none() {
            return Err(Box::new(Error::PublishMissingSource));
        }

        // verify internal design unit visibility
        crate::info!("verifying source file visibility ...");
        if let Err(e) = Self::check_design_unit_visibility_okay(&local_ip, priv_by_def, all_pub) {
            return Err(Box::new(Error::PublishUnitVisibilityFailed(LastError(
                e.to_string(),
            ))))?;
        }

        // verify the graph build with no errors
        crate::info!("verifying hardware graph construction ...");
        if let Err(e) = Self::check_graph_builds_okay(&local_ip, &catalog, priv_by_def) {
            return Err(Box::new(Error::PublishHdlGraphFailed(LastError(
                e.to_string(),
            ))))?;
        }

        Ok(())
    }

    pub fn test_download_and_install(
        local_ip: &Project,
        c: &Context,
        remove: bool,
        verbose: bool,
    ) -> Result<Option<Changes>, Fault> {
        let verbose_install = remove == false && verbose == true;

        // install from local path to what its checksum would be
        let local_sum = {
            let local_install = Install::install(local_ip, c.get_cache_path(), true, false)?
                .expect("project should be installed from local");
            let sum = Project::compute_checksum(&local_install.get_root());
            Remove::remove_install(&local_install)?;
            sum
        };

        let ip = local_ip.get_man().get_project();
        let src = ip.get_source().as_ref().unwrap();
        // get the ip from the internet and as an archive
        let bytes = Install::download_target_from_url(
            c,
            &src.get_url(),
            &Some(ip.into_project_id_spec().to_partial_project_id_spec()),
            true,
        )?
        .1;
        // try to extract the ip from the archives
        let tmp_archive_staging_dir = tempfile::tempdir()?.into_path();
        if let Err(e) = ProjectArchive::extract(&bytes, &tmp_archive_staging_dir) {
            fs::remove_dir_all(tmp_archive_staging_dir)?;
            return Err(e);
        }
        let unzipped_ip = match Project::load(tmp_archive_staging_dir.clone(), false, false) {
            Ok(x) => x,
            Err(e) => {
                fs::remove_dir_all(tmp_archive_staging_dir)?;
                return Err(e);
            }
        };
        // try to install the ip
        let installed_ip =
            match Install::install(&unzipped_ip, c.get_cache_path(), true, verbose_install) {
                Ok(x) => {
                    fs::remove_dir_all(tmp_archive_staging_dir)?;
                    x.expect("project should be installed from archive")
                }
                Err(e) => {
                    fs::remove_dir_all(tmp_archive_staging_dir)?;
                    return Err(e);
                }
            };
        // after collecting the checksums, clean up the installations if needed
        let installed_sum = Project::compute_checksum(installed_ip.get_root());

        if remove == true {
            Remove::remove_download(c.get_downloads_path(), &unzipped_ip)?;
            Remove::remove_install(&installed_ip)?;
        }

        match local_sum == installed_sum {
            true => Ok(match remove {
                true => None,
                false => Some(Changes {
                    downloads_path: c.get_downloads_path().clone(),
                    archived_ip: unzipped_ip,
                    cached_ip: installed_ip,
                }),
            }),
            false => {
                // make sure files are deleted
                if remove == false {
                    Remove::remove_download(c.get_downloads_path(), &unzipped_ip)?;
                    Remove::remove_install(&installed_ip)?;
                }
                Err(Error::PublishChecksumsOff(Hint::PublishSyncRemote))?
            }
        }
    }

    /// Verifies the design units for this ip will be found okay
    pub fn check_design_unit_visibility_okay(
        local_ip: &Project,
        priv_by_def: bool,
        all_pub: bool,
    ) -> Result<(), Fault> {
        // Verify design units are not empty
        if local_ip.found_zero_units()? == true {
            return Err(Box::new(Error::IpZeroDesignUnitsFound(
                Hint::AddSourceFiles,
            )));
        }
        // Verify we have public design units
        if local_ip.found_all_units_private(priv_by_def)? == true {
            return Err(Box::new(Error::IpNoDesignUnitsWithPublic(
                Hint::FixPublicEntry,
            )));
        }
        // Verify we are not all assumed private
        if local_ip.force_all_units_private(priv_by_def) == true && all_pub == false {
            return Err(Box::new(Error::IpAssumedAllPrivateByDefault(
                Hint::AddPublicEntry,
            )));
        }
        Ok(())
    }

    /// Verifies that we can build the graph for this ip `local_ip` without errors.
    pub fn check_graph_builds_okay(
        local_ip: &Project,
        catalog: &Catalog,
        priv_by_def: bool,
    ) -> Result<(), Fault> {
        let ip_graph = algo::compute_final_project_graph(&local_ip, Some(&catalog), priv_by_def)?;
        let files = algo::build_project_file_list(&ip_graph, &local_ip);
        let _global_graph = Plan::build_full_graph(&files)?;
        Ok(())
    }

    fn publish_all(
        &self,
        local_ip: &Project,
        channels: HashMap<&String, &Channel>,
        mut env: Environment,
        changes: &Option<Changes>,
    ) -> Result<(), Fault> {
        // publish to each channel
        for (name, chan) in &channels {
            crate::info!("publishing to {:?} channel ...", name);
            // update the index path
            let index_dir = Self::create_pointer_directory(&local_ip);
            let index_path = filesystem::into_std_str(chan.get_root().join(index_dir));
            // set the CHANNEL environment variables
            env = env.overwrite(EnvVar::with(ORBIT_CHANNEL_NAME, chan.get_name()));
            env = env.overwrite(EnvVar::with(
                ORBIT_CHANNEL_DIR,
                &filesystem::into_std_str(chan.get_root().to_path_buf()),
            ));
            env = env.overwrite(EnvVar::with(ORBIT_CHANNEL_PROJECT_DIR, index_path.as_str()));
            // publish to this channel
            match self.publish(local_ip, chan, &env) {
                Ok(_) => (),
                Err(e) => {
                    self.rollback_changes(local_ip, chan, changes)?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn publish(
        &self,
        local_ip: &Project,
        channel: &Channel,
        env: &Environment,
    ) -> Result<(), Fault> {
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
    /// The directory is something like this: `uuid[0]/uuid-version`.
    fn create_pointer_directory(ip: &Project) -> PathBuf {
        let name = ip.get_man().get_project().get_name();
        let version = ip.get_man().get_project().get_version();
        let uuid = ip.get_uuid();
        PathBuf::new()
            .join(String::from(uuid.encode().chars().next().unwrap()))
            .join(PointerSlot::new(name, uuid, version).as_ref())
    }

    /// Writes the ip's manifest to the channel.
    fn copy_to_channel(&self, local_ip: &Project, channel: &Channel) -> Result<(), Fault> {
        let output_dir = Self::create_pointer_directory(&local_ip);
        let output_path = channel.get_root().join(output_dir);
        // create any mising directories
        std::fs::create_dir_all(&output_path)?;
        // copy the (raw) manifest there (in formatted string)
        std::fs::write(
            output_path.join(PROJECT_MANIFEST_FILE),
            local_ip.get_man().to_string(),
        )?;
        // copy the (raw) lockfile there
        local_ip.get_lock().save_to_disk(&output_path)?;
        // create a JSON metadata file there
        let json_data = JsonMeta::new(local_ip)?;
        std::fs::write(
            output_path.join(PROJECT_JSON_FILE),
            serde_json::to_string_pretty(&json_data)?,
        )?;
        Ok(())
    }

    /// Rollback the progress made during publish if an error has occurred.
    ///
    /// This function should be called before returning the final error from the publish
    /// operation to allow users to try again from a known state.
    fn rollback_changes(
        &self,
        local_ip: &Project,
        channel: &Channel,
        changes: &Option<Changes>,
    ) -> Result<(), Fault> {
        // remove the installation and download
        if self.no_install == false {
            if let Some(changes) = changes {
                Remove::remove_download(&changes.downloads_path, &changes.archived_ip)?;
                Remove::remove_install(&changes.cached_ip)?;
            }
        }

        let index_dir = Self::create_pointer_directory(&local_ip);
        let index_path = channel.get_root().join(index_dir);

        if index_path.exists() && index_path.is_dir() {
            std::fs::remove_dir_all(index_path)?;
        }
        // check if we should remove the first-layer directory
        let first_dir = PathBuf::from(String::from(
            local_ip
                .get_man()
                .get_project()
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

pub struct Changes {
    pub downloads_path: PathBuf,
    pub archived_ip: Project,
    pub cached_ip: Project,
}
