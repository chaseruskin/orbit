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
use crate::commands::plan::Plan;
use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::ip;
use crate::core::lockfile::LockFile;
use crate::core::manifest;
use crate::core::v2::manifest::IP_MANIFEST_FILE;
use crate::core::v2::manifest::Manifest;
use crate::core::v2::manifest::FromFile;
use clif::Cli;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::{AnyError, Fault};
use crate::core::version::AnyVersion;
use tempfile::TempDir;
use std::path::PathBuf;
use crate::core::extgit::ExtGit;
use crate::OrbitResult;
use crate::util::filesystem::Standardize;
use crate::core::v2::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct Install {
    path: PathBuf,
    force: bool,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Install {
            force: cli.check_flag(Flag::new("force"))?,
            path: cli.check_option(Optional::new("path"))?.unwrap_or(PathBuf::from(".")),
        });
        command
    }
}

impl Command<Context> for Install {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // verify the path points to a valid ip
        let dest = PathBuf::standardize(self.path.clone());

        Ip::is_valid(&dest)?;

        // todo!();
        
        // let temporary directory exist for lifetime of install in case of using it
        // let temp_dir = tempdir()?;

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

/// Grabs the root path to the repository to perform the installation on.
pub fn fetch_install_path(ip: &PkgId, catalog: &Catalog, disable_ssh: bool, temp_dir: &TempDir) -> Result<PathBuf, Fault> {
    let ids = catalog.inner().keys().map(|f| { f }).collect();

    let target = crate::core::ip::find_ip(ip, ids)?;
    // gather all possible versions found for this IP
    let status = catalog.inner().get(&target).take().unwrap();

    // check the store/ for the repository
    if let Some(root) = catalog.get_store().as_stored(&target) {
        Ok(root)
    // clone from remote repository if exists
    } else if let Some(url) = status.try_repository() {
        let path = temp_dir.path().to_path_buf();
        println!("info: fetching repository ...");
        ExtGit::new(None).clone(&url, &path, disable_ssh)?;
        Ok(path)
    } else {
        // @TODO last resort, clone the actual dev directory to a temp folder
        panic!("no repository to access ip")
    }
}

impl Install {

    pub fn install_from_lock_file(&self, lock: &LockFile, catalog: &Catalog) -> Result<(), Fault> {
        // build entire dependency graph from lockfile @todo: denote which ip's are from dev path to ensure they are "develop_from_lock_entry"
        let graph = ip::graph_ip_from_lock(&lock)?;
        // sort to topological ordering
        let mut order = graph.get_graph().topological_sort();
        // remove target ip from the list of intermediate installations
        order.pop();

        for i in order {
            let entry = graph.get_node_by_index(i).unwrap().as_ref();
            // check if already installed
            match std::path::Path::exists(&catalog.get_cache_path().join(entry.to_cache_slot().as_ref())) {
                true => println!("info: {} v{} already installed", entry.get_name(), entry.get_version()),
                false => Plan::install_from_lock_entry(entry, &AnyVersion::Specific(entry.get_version().to_partial_version()), &catalog, false)?,
            }
        }
        Ok(())
    }

    // /// Searches through a given root as a git repository to find a tagged commit
    // /// matching `version` with highest compatibility and contains a manifest.
    // fn detect_manifest(root: &PathBuf, version: &AnyVersion, store: &Store) -> Result<IpManifest, Fault>{
    //     let repo = Repository::open(&root)?;

    //     // find the specified version for the given ip
    //     let space = ExtGit::gather_version_tags(&repo)?;
    //     let version_space: Vec<&Version> = space.iter().collect();
    //     let version = match version::get_target_version(&version, &version_space) {
    //         Ok(r) => r,
    //         // update store if it was used
    //         Err(e) => {
    //             if store.is_path_in_store(&root) && ExtGit::is_remote_linked(&repo)? == true {
    //                 println!("info: could not find a version tag matching '{}' in stored repository", &version);
    //                 println!("info: pulling latest to store ...");
    //                 ExtGit::new(None).path(root.to_path_buf()).pull()?;
    //                 ExtGit::gather_version_tags(&repo)?;
    //                 version::get_target_version(&version, &version_space)?
    //             } else {
    //                 return Err(e)?
    //             }
    //         }
    //     };

    //     ExtGit::checkout_tag_state(&repo, &version)?;

    //     // make an ip manifest
    //     Ok(IpManifest::from_path(&root)?)
    // }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    /// 
    /// Errors if the ip is already installed unless `force` is true.
    pub fn install(installation_path: &PathBuf, cache_root: &std::path::PathBuf, force: bool) -> Result<(), Fault> {
        // make an ip manifest
        // let ip = Self::detect_manifest(&installation_path, &version, &store)?;
        let man = Manifest::from_file(&installation_path.join(IP_MANIFEST_FILE))?;
        let target = man.get_ip().get_name();

        // move into stored directory to compute checksum for the tagged version
        let src = installation_path;
        // let temp = match store.is_stored(&target) {
        //     true => ip.get_root(),
        //     // throw repository into the store/ for future use
        //     false => store.store(&ip)?,
        // };
        // update version to be a specific complete spec
        let version = man.get_ip().get_version();

        // let repo = Repository::open(&temp)?;
        // ExtGit::checkout_tag_state(&repo, &version)?;

        println!("info: installing {} v{} ...", target, version);

        // perform sha256 on the temporary cloned directory 
        let checksum = Ip::compute_checksum(&installation_path);
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
        // copy contents into cache slot
        crate::util::filesystem::copy(&src, &cache_slot, true)?;
        // // revert the store back to its HEAD
        // ExtGit::checkout_head(&repo)?;

        // write the checksum to the directory
        std::fs::write(&cache_slot.join(manifest::ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        // write the metadata to the directory
        // let mut installed_ip = IpManifest::from_path(&cache_slot)?;

        // let installed_man = Manifest::from_
        // installed_ip.write_metadata()?;
        Ok(())
    }

    fn run(&self, installation_path: &PathBuf, catalog: &Catalog) -> Result<(), Fault> {
        // check if there is a potential lockfile to use
        let _man = Manifest::from_file(&installation_path.join(IP_MANIFEST_FILE))?;
        // if let Some(lock) = man.get_lockfile() {
        //     Self::install_from_lock_file(&self, &lock, &catalog)?;
        // }
        // if the lockfile is invalid, then it will only install the current request and zero dependencies
        
        let _ = Self::install(&installation_path, &catalog.get_cache_path(), self.force)?;
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

Use 'orbit help install' to learn more about the command.
";