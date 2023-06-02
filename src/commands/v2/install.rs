//! The installation process:
//! 1. Optionally ask for an ip to install (default: current working ip)
//! --!-- Get the folder and change directories to the folder requiring installation --!--
//! * at the point in the process when the program is at the folder, it is assumed all sub-deps are also already installed
//! 2. Write results from computing the available units for the package
//! 3. Write results for information about accessing C,I,S,A
//! 3. Verify a .lock file is available (is this needed? - do dependents even read from this?)
//! 4. Move only relevant files to artifact directory (no .git/, etc.)
//! 5. Compute checksum on entire directory
//! 6. Zip contents and store in "store" for future re-installation
//! 7. Place artifact directory in "cache" for catalog lookup
//! 
//! One issue that remains is how to retrieve packages from online automatically.
//! 
//! The download process:
//!     - write a lockfile
//!     - ...
//! 

use clif::cmd::{FromCli, Command};
use crate::core::v2::catalog::CacheSlot;
use crate::core::v2::catalog::Catalog;
use crate::core::v2::manifest::ORBIT_SUM_FILE;
use crate::util::filesystem;
use clif::Cli;
use std::env::current_dir;
use std::fs;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::Fault;
use std::path::PathBuf;
use crate::OrbitResult;
use crate::util::filesystem::Standardize;

use crate::core::v2::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct Install {
    path: PathBuf,
    force: bool,
    deps_only: bool,
    all: bool,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Install {
            force: cli.check_flag(Flag::new("force"))?,
            deps_only: cli.check_flag(Flag::new("deps"))?,
            all: cli.check_flag(Flag::new("all"))?,
            path: cli.check_option(Optional::new("path"))?.unwrap_or(PathBuf::from(".")),
        });
        command
    }
}

use crate::core::v2::algo;
use crate::core::v2::lockfile::LockFile;

impl Command<Context> for Install {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // verify the path points to a valid ip
        let path = filesystem::resolve_rel_path(&current_dir().unwrap(), &filesystem::into_std_str(self.path.clone()));
        let dest = PathBuf::standardize(PathBuf::from(path));

        Ip::is_valid(&dest)?;
        
        // gather the catalog (all manifests)
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .queue(c.get_queue_path())?;

        // @todo: check if there is a potential lockfile to use
        let target = Ip::load(dest.clone())?;

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if target.can_use_lock() == true && self.force == false {
            let le = LockEntry::from((&target, true));

            let lf = {
                let lf = target.get_lock().clone();
                // find the dev-deps and remove them from the lockfile data
                let entries: Vec<LockEntry> = match self.all {
                    // install dev-deps anyway
                    true => lf.unwrap(),
                    // do not install dev-deps (filter them out)
                    false => {
                        lf.unwrap().into_iter().filter(|p| match target.get_man().get_dev_deps().get(p.get_name()) {
                            Some(v) => if p.get_version() == v { false } else { true },
                            None => true,
                        }).collect()
                    }
                };
                LockFile::wrap(entries)
            };
            
            plan::download_missing_deps(&lf, &le, &catalog, &c.get_config().get_protocols())?;
            // recollect the queued items to update the catalog
            catalog = catalog
                .installations(c.get_cache_path())?
                .queue(c.get_queue_path())?;

            plan::install_missing_deps(&lf, &le, &catalog)?;
            // recollect the installations and queued items to update the catalog
            catalog = catalog
                .installations(c.get_cache_path())?
                .queue(c.get_queue_path())?;
        }
        // generate lock file if it is missing
        if target.lock_exists() == false {
            // build entire ip graph and resolve with dynamic symbol transformation
            let ip_graph = algo::compute_final_ip_graph(&target, &catalog)?;
            Plan::write_lockfile(&target, &ip_graph, true)?;
        }
        // install the top-level target
        self.run(&target, &catalog)
    }
}

use crate::core::v2::lockfile::LockEntry;
use crate::commands::v2::plan;

use super::plan::Plan;

impl Install {

    pub fn is_checksum_good(root: &PathBuf) -> bool {
        // verify the checksum
        if let Some(sha) = Ip::read_checksum_proof(&root) {
            // make sure the sums match expected
            sha == Ip::compute_checksum(&root)
        } else {
            // failing to compute a checksum
            false
        }
    }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    /// 
    /// Returns `true` if the IP was successfully installed and `false` if it already existed.
    pub fn install(src: &Ip, cache_root: &std::path::PathBuf, force: bool) -> Result<bool, Fault> {
        // temporary destination to move files for processing and manipulation
        let dest = tempfile::tempdir()?.into_path();
        filesystem::copy(src.get_root(), &dest, true)?;

        // @todo: listing all units

        // @todo: store a LUT for unit names to the correct file to read when computing "get" command

        // @todo: getting the size of the entire directory

        // access the name and version
        let version = src.get_man().get_ip().get_version();
        let target = src.get_man().get_ip().get_name();
        let ip_spec = src.get_man().get_ip().into_ip_spec();
        println!("info: Installing IP {} ...", &ip_spec);

        // perform sha256 on the temporary cloned directory 
        let checksum = Ip::compute_checksum(&dest);
        // println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = CacheSlot::new(target, &version, &checksum);
        let cache_slot = cache_root.join(&cache_slot_name.as_ref());
        // check if the slot is occupied in the cache
        if cache_slot.exists() == true {
            // check if we should proceed with force regardless if the installation is valid
            if force == true {
                std::fs::remove_dir_all(&cache_slot)?;
            } else {
                // ip is already installed
                if Self::is_checksum_good(&cache_slot) == true {
                    // clean up the temporary directory ourself
                    fs::remove_dir_all(dest)?;
                    return Ok(false)
                } else {
                    println!("info: Reinstalling IP {} due to bad checksum ...", ip_spec);

                    // blow directory up for re-install
                    std::fs::remove_dir_all(&cache_slot)?;
                }
            }
        }
        // copy contents into cache slot from temporary destination
        crate::util::filesystem::copy(&dest, &cache_slot, false)?;
 
        // clean up the temporary directory ourself
        fs::remove_dir_all(dest)?;

        // write the checksum to the directory (this file is excluded from auditing)
        std::fs::write(&cache_slot.join(ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        
        Ok(true)
    }

    fn run(&self, target: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        let result = Self::install(&target, &catalog.get_cache_path(), self.force)?;

        if result == false {
            println!("info: IP {} is already installed", target.get_man().get_ip().into_ip_spec());
        }

        Ok(())
        // store results from expensive computations into specific orbit files

        // print download list for top-level package
        // if self.compile == true {
        //     for s in Self::compile_download_list(ip.get_lock(), &catalog, false) {
        //         println!("{}", s);
        //     }
        //     return Ok(())
        // }

        // _pkg.get_lock().save_to_disk(&_pkg.get_root())?;
        // todo!();
        
        // @todo: check lockfile to process installing any IP that may be already downloaded to the queue

        // verify each requirement for the IP is also installed (o.w. install)
        

        // if let Some(lock) = man.get_lockfile() {
        //     Self::install_from_lock_file(&self, &lock, &catalog)?;
        // }
        // if the lockfile is invalid, then it will only install the current request and zero dependencies
    }
}

const HELP: &str = "\
Places an immutable version of an ip to the cache for dependency usage.

Usage:
    orbit install [options]

Options:
    --path <path>           destination directory to install into the cache
    --ip <name>             the ip to match for install into the cache
    --force                 install regardless of cache slot occupancy
    --all                   install all dependencies including development

Use 'orbit help install' to learn more about the command.
";