use super::helps::remove;
use crate::core::catalog::{CacheSlot, Catalog};
use crate::core::context::Context;
use crate::core::ip::{Ip, PartialIpSpec};
use crate::core::version::AnyVersion;
use crate::util::anyerror::AnyError;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use cliproc::{cli, proc};
use cliproc::{Cli, Flag, Help, Positional, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Remove {
    ip: PartialIpSpec,
    all: bool,
    recurse: bool,
    // @todo: add option to remove all versions (including store)
    // @todo:
}

impl Subcommand<Context> for Remove {
    fn construct<'c>(cli: &'c mut Cli) -> cli::Result<Self> {
        cli.check_help(Help::default().text(remove::HELP))?;
        Ok(Remove {
            all: cli.check_flag(Flag::new("all"))?,
            recurse: cli.check_flag(Flag::new("recurse"))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        if self.recurse == true {
            todo!("implement recursive removal")
        }

        // collect the catalog from dev and installations
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // check for ip in development or installation
        let status = match catalog.inner().get(&self.ip.get_name()) {
            Some(st) => st,
            None => return Err(AnyError(format!("ip '{}' does not exist", self.ip)))?,
        };

        // determine the ip version (invariant of state) that matches
        let detected_version = {
            let install_version = status.get_install(&self.ip.get_version());
            let download_version = status.get_download(&self.ip.get_version());

            if let Some(iv) = install_version {
                if let Some(dv) = download_version {
                    if iv.get_man().get_ip().get_version() > dv.get_man().get_ip().get_version() {
                        AnyVersion::Specific(
                            iv.get_man().get_ip().get_version().to_partial_version(),
                        )
                    } else {
                        AnyVersion::Specific(
                            dv.get_man().get_ip().get_version().to_partial_version(),
                        )
                    }
                } else {
                    AnyVersion::Specific(iv.get_man().get_ip().get_version().to_partial_version())
                }
            } else if let Some(dv) = download_version {
                AnyVersion::Specific(dv.get_man().get_ip().get_version().to_partial_version())
            } else {
                self.ip.get_version().clone()
            }
        };

        // grab the ip's manifest
        match status.get_install(&detected_version) {
            Some(t) => {
                self.remove_install(t)?;
                self.remove_dynamics(c.get_cache_path(), t)?;
            }
            None => match self.all {
                true => {
                    println!("info: ip {} already removed from cache", self.ip);
                }
                false => {
                    return Err(AnyError(format!(
                        "ip {} does not exist in the cache",
                        self.ip
                    )))?
                }
            },
        };

        // delete the project from downloads (if requested)
        if self.all == true {
            // grab the ip's manifest
            let target = match status.get_download(&detected_version) {
                Some(t) => t,
                None => {
                    return Err(AnyError(format!(
                        "ip {} does not exist as download",
                        self.ip
                    )))?
                }
            };
            let ip_spec = target.get_man().get_ip().into_ip_spec();
            // delete the project from the cache (default behavior)
            fs::remove_file(
                c.get_downloads_path().join(
                    &target
                        .get_lock()
                        .get_self_entry(&ip_spec.get_name())
                        .unwrap()
                        .to_download_slot_key()
                        .as_ref(),
                ),
            )?;
            println!("info: Removed ip {} from downloads", ip_spec);
        }

        self.run()
    }
}

impl Remove {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Removes the installed IP from its root directory. This function assumes
    /// the `target` IP exists under the installation path (cache path).
    fn remove_install(&self, target: &Ip) -> Result<(), Box<dyn Error>> {
        let ip_spec = target.get_man().get_ip().into_ip_spec();

        // delete the project from the cache (default behavior)
        fs::remove_dir_all(target.get_root())?;
        println!("info: Removed ip {} from the cache", ip_spec);
        Ok(())
    }

    fn remove_dynamics(&self, cache_path: &PathBuf, target: &Ip) -> Result<(), Box<dyn Error>> {
        let ip_spec = target.get_man().get_ip().into_ip_spec();

        // check for any "dynamics" under this target
        for dir in fs::read_dir(cache_path)? {
            // only check valid directory entries
            if let Ok(entry) = dir {
                // println!("{:?}", entry.file_name());
                let file_name = entry.file_name();
                if let Some(cache_slot) = CacheSlot::try_from_str(&file_name.to_string_lossy()) {
                    // check if the slot is matching
                    if target.get_man().get_ip().get_name() != cache_slot.get_name()
                        || target.get_man().get_ip().get_version() != cache_slot.get_version()
                    {
                        continue;
                    }
                    // check for same UUID
                    let cached_ip = Ip::load(entry.path().to_path_buf(), false)?;
                    if cached_ip.get_uuid() != target.get_uuid() {
                        continue;
                    }
                    // remove the slot if it is dynamic
                    if cached_ip.is_dynamic() == true {
                        fs::remove_dir_all(entry.path())?;
                        println!(
                            "info: Removed a dynamic variant of ip {} from the cache",
                            ip_spec
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
