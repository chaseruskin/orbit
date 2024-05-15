use crate::core::catalog::{CacheSlot, Catalog};
use crate::core::context::Context;
use crate::core::ip::{Ip, PartialIpSpec};
use crate::util::anyerror::AnyError;
use crate::OrbitResult;
use clif::arg::{Flag, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Uninstall {
    ip: PartialIpSpec,
    full: bool,
    // @todo: add option to remove all versions (including store)
    // @todo:
}

impl FromCli for Uninstall {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Uninstall {
            full: cli.check_flag(Flag::new("full"))?,
            ip: cli.require_positional(Positional::new("spec"))?,
        });
        command
    }
}

impl Command<Context> for Uninstall {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // collect the catalog from dev and installations
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // check for ip in development or installation
        let status = match catalog.inner().get(&self.ip.get_name()) {
            Some(st) => st,
            None => {
                return Err(AnyError(format!(
                    "ip '{}' does not exist in the cache",
                    self.ip
                )))?
            }
        };

        // grab the ip's manifest
        let target = match status.get_install(&self.ip.get_version()) {
            Some(t) => t,
            None => {
                return Err(AnyError(format!(
                    "IP {} does not exist in the cache",
                    self.ip
                )))?
            }
        };

        let ip_spec = target.get_man().get_ip().into_ip_spec();

        // delete the project
        fs::remove_dir_all(target.get_root())?;
        println!("info: Removed IP {} from the cache", ip_spec);

        // check for any "dynamics" under this target
        for dir in fs::read_dir(c.get_cache_path())? {
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
                            "info: Removed a dynamic variant of IP {} from the cache",
                            ip_spec
                        );
                    }
                }
            }
        }

        self.run()
    }
}

impl Uninstall {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Remove an ip from the catalog.

Usage:
    orbit uninstall [options] <ip>

Args:
    <spec>      the name corresponding to the ip to delete
    --full      fully remove the ip and its dependencies

Use 'orbit help uninstall' to learn more about the command.
";
