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
//! 
//! 

use clif::cmd::{FromCli, Command};
use crate::core::v2::catalog::CacheSlot;
use crate::core::v2::catalog::Catalog;
use crate::core::v2::manifest::{IP_MANIFEST_FILE, ORBIT_SUM_FILE};
use crate::core::v2::manifest::Manifest;
use crate::core::v2::manifest::FromFile;
use crate::util::filesystem;
use clif::Cli;
use std::env::current_dir;
use std::fs;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::{AnyError, Fault};
use std::path::PathBuf;
use crate::OrbitResult;
use crate::util::filesystem::Standardize;

use crate::core::v2::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct Install {
    path: PathBuf,
    force: bool,
    compile: bool,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Install {
            force: cli.check_flag(Flag::new("force"))?,
            compile: cli.check_flag(Flag::new("compile"))?,
            path: cli.check_option(Optional::new("path"))?.unwrap_or(PathBuf::from(".")),
        });
        command
    }
}

impl Command<Context> for Install {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // verify the path points to a valid ip
        let path = filesystem::resolve_rel_path(&current_dir().unwrap(), &filesystem::into_std_str(self.path.clone()));
        let dest = PathBuf::standardize(PathBuf::from(path));

        Ip::is_valid(&dest)?;
        
        // gather the catalog (all manifests)
        let catalog = Catalog::new()
            // .store(c.get_store_path())
            // .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?;
            // .available(c.get_vendors())?;

        // enter action
        self.run(&dest, &catalog)
    }
}

impl Install {

    // pub fn install_from_lock_file(&self, lock: &LockFile, catalog: &Catalog) -> Result<(), Fault> {
    //     // build entire dependency graph from lockfile @todo: denote which ip's are from dev path to ensure they are "develop_from_lock_entry"
    //     let graph = ip::graph_ip_from_lock(&lock)?;
    //     // sort to topological ordering
    //     let mut order = graph.get_graph().topological_sort();
    //     // remove target ip from the list of intermediate installations
    //     order.pop();

    //     for i in order {
    //         let entry = graph.get_node_by_index(i).unwrap().as_ref();
    //         // check if already installed
    //         match std::path::Path::exists(&catalog.get_cache_path().join(entry.to_cache_slot().as_ref())) {
    //             true => println!("info: {} v{} already installed", entry.get_name(), entry.get_version()),
    //             false => Plan::install_from_lock_entry(entry, &AnyVersion::Specific(entry.get_version().to_partial_version()), &catalog, false)?,
    //         }
    //     }
    //     Ok(())
    // }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    /// 
    /// Errors if the ip is already installed unless `force` is true.
    pub fn install(src: &PathBuf, cache_root: &std::path::PathBuf, force: bool) -> Result<(), Fault> {
        // @todo: validate the IP and its dependencies
        let man = Manifest::from_file(&src.join(IP_MANIFEST_FILE))?;

        // temporary destination to move files for processing and manipulation
        let dest = tempfile::tempdir()?.into_path();
        filesystem::copy(src, &dest, true)?;

        // @todo: listing all units

        // @todo: store a LUT for unit names to the correct file to read when computing "get" command

        // @todo: getting the size of the entire directory

        // access the name and version
        let version = man.get_ip().get_version();
        let target = man.get_ip().get_name();
        println!("info: installing {} v{} ...", target, version);

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
                // verify the installed version is valid
                if let Some(sha) = Ip::read_checksum_proof(&cache_slot) {
                    // recompute the checksum on the cache installation
                    if sha == Ip::compute_checksum(&cache_slot) {
                        return Err(AnyError(format!("ip '{}' as version '{}' is already installed", target, version)))?
                    }
                }
                println!("info: reinstalling ip '{}' as version '{}' due to bad checksum", target, version);

                // blow directory up for re-install
                std::fs::remove_dir_all(&cache_slot)?;
            }
        }
        // copy contents into cache slot from temporary destination
        crate::util::filesystem::copy(&dest, &cache_slot, false)?;
 
        // clean up the temporary directory ourself
        fs::remove_dir_all(dest)?;

        // write the checksum to the directory (this file is excluded from auditing)
        std::fs::write(&cache_slot.join(ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        
        Ok(())
    }

    fn run(&self, src: &PathBuf, catalog: &Catalog) -> Result<(), Fault> {
        // @todo: check if there is a potential lockfile to use
        let _ip = Ip::load(src.clone())?;

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

        // if let Some(lock) = man.get_lockfile() {
        //     Self::install_from_lock_file(&self, &lock, &catalog)?;
        // }
        // if the lockfile is invalid, then it will only install the current request and zero dependencies
        
        let _ = Self::install(&src, &catalog.get_cache_path(), self.force)?;
        Ok(())
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
    --compile               gather the list of necessary dependencies to download

Use 'orbit help install' to learn more about the command.
";