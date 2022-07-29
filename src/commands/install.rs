use crate::Command;
use crate::FromCli;
use crate::commands::plan::Plan;
use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::ip;
use crate::core::lockfile::LockFile;
use crate::core::manifest;
use crate::core::manifest::IpManifest;
use crate::core::version;
use crate::interface::cli::Cli;
use crate::interface::arg::{Optional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::core::version::Version;
use crate::util::anyerror::{AnyError, Fault};
use crate::core::version::AnyVersion;
use crate::util::url::Url;
use colored::Colorize;
use git2::Repository;
use tempfile::TempDir;
use tempfile::tempdir;
use crate::core::store::Store;
use std::path::PathBuf;
use crate::core::extgit::ExtGit;

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: Option<PkgId>,
    path: Option<std::path::PathBuf>,
    git: Option<Url>,
    version: AnyVersion,
    disable_ssh: bool,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Install {
            git: cli.check_option(Optional::new("git").value("url"))?,
            path: cli.check_option(Optional::new("path"))?,
            version: cli.check_option(Optional::new("ver").switch('v'))?.unwrap_or(AnyVersion::Latest),
            ip: cli.check_option(Optional::new("ip"))?,
            disable_ssh: cli.check_flag(Flag::new("disable-ssh"))?,
        });
        command
    }
}

impl Command for Install {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // verify user is not requesting the dev version to be installed
        match &self.version {
            AnyVersion::Dev => return Err(AnyError(format!("{}", "a dev version cannot be installed to the cache")))?,
            _ => ()
        };
        // let temporary directory exist for lifetime of install in case of using it
        let temp_dir = tempdir()?;

        // gather the catalog (all manifests)
        let catalog = Catalog::new()
            .store(c.get_store_path())
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(c.get_vendors())?;

        // get to the repository (root path)
        let ip_root = if let Some(ip) = &self.ip {

            // grab install path
            fetch_install_path(ip, &catalog, self.disable_ssh, &temp_dir)?
        } else if let Some(url) = &self.git {
            // clone from remote repository
            let path = temp_dir.path().to_path_buf();
            println!("info: fetching repository ...");
            ExtGit::new(None).clone(url, &path, self.disable_ssh)?;
            path
        } else if let Some(path) = &self.path {
            // traverse filesystem
            path.clone()
        } else {
            return Err(AnyError(format!("select an option to install from '{}', '{}', or '{}'", "--ip".yellow(), "--git".yellow(), "--path".yellow())))?
        };
        // enter action
        self.run(&ip_root, &catalog, c.force)
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
        // build entire dependency graph from lockfile
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
                false => Plan::install_from_lock_entry(entry, &AnyVersion::Specific(entry.get_version().to_partial_version()), &catalog, self.disable_ssh)?,
            }
        }
        Ok(())
    }

    /// Searches through a given root as a git repository to find a tagged commit
    /// matching `version` with highest compatibility and contains a manifest.
    fn detect_manifest(root: &PathBuf, version: &AnyVersion) -> Result<IpManifest, Fault>{
        let repo = Repository::open(&root)?;

        // find the specified version for the given ip
        let space = ExtGit::gather_version_tags(&repo)?;
        let version_space: Vec<&Version> = space.iter().collect();
        let version = version::get_target_version(&version, &version_space)?;

        ExtGit::checkout_tag_state(&repo, &version)?;

        // make an ip manifest
        Ok(IpManifest::from_path(&root)?)
    }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    /// 
    /// Errors if the ip is already installed unless `force` is true.
    pub fn install(installation_path: &PathBuf, version: &AnyVersion, cache_root: &std::path::PathBuf, force: bool, store: &Store) -> Result<IpManifest, Fault> {
        // make an ip manifest
        let ip = Self::detect_manifest(&installation_path, &version)?;
        let target = ip.get_pkgid();

        // move into stored directory to compute checksum for the tagged version
        let temp = match store.is_stored(&target) {
            true => ip.get_root(),
            // throw repository into the store/ for future use
            false => store.store(&ip)?,
        };
        // update version to be a specific complete spec
        let version = ip.get_version();

        let repo = Repository::open(&temp)?;
        ExtGit::checkout_tag_state(&repo, &version)?;

        let root = IpManifest::from_path(&temp).unwrap();
        println!("info: installing {} v{} ...", root.get_pkgid(), root.get_version());

        // perform sha256 on the temporary cloned directory 
        let checksum = root.compute_checksum();
        // println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = CacheSlot::new(target.get_name(), &version, &checksum);
        let cache_slot = cache_root.join(&cache_slot_name.as_ref());
        if std::path::Path::exists(&cache_slot) == true {
            // check if we should proceed with force regardless if the installation is valid
            if force == true {
                std::fs::remove_dir_all(&cache_slot)?;
            } else {
                let cached_ip = IpManifest::from_path(&cache_slot)?;
                // verify the installed version is valid
                if let Some(sha) = cached_ip.read_checksum_proof() {
                    // recompute the checksum on the cache installation
                    if sha == cached_ip.compute_checksum() {
                        return Err(AnyError(format!("ip '{}' as version '{}' is already installed", target, version)))?
                    }
                }
                println!("info: reinstalling ip '{}' as version '{}' due to bad checksum", target, version);

                // blow directory up for re-install
                std::fs::remove_dir_all(&cache_slot)?;
            }
        }
        // copy contents into cache slot
        crate::util::filesystem::copy(&temp, &cache_slot, true)?;
        // revert the store back to its HEAD
        ExtGit::checkout_head(&repo)?;
        
        // write the checksum to the directory
        std::fs::write(&cache_slot.join(manifest::ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        // write the metadata to the directory
        let mut installed_ip = IpManifest::from_path(&cache_slot)?;
        installed_ip.write_metadata()?;
        Ok(installed_ip)
    }

    fn run(&self, installation_path: &PathBuf, catalog: &Catalog, force: bool, ) -> Result<(), Fault> {
        // check if there is a potential lockfile to use
        if let Ok(manifest) = Self::detect_manifest(&installation_path, &self.version) {
            if let Some(lock) = manifest.get_lockfile() {
                Self::install_from_lock_file(&self, &lock, &catalog)?;
            }
            // if the lockfile is invalid, then it will only install the current request and zero dependencies
        }
        let _ = Self::install(&installation_path, &self.version, &catalog.get_cache_path(), force, &catalog.get_store())?;
        Ok(())
    }
}

const HELP: &str = "\
Places an immutable version of an ip to the cache for dependency usage.

Usage:
    orbit install [options]

Options:
    --ip <ip>               pkgid to access an orbit ip to install
    --ver, -v <version>     version to install
    --path <path>           local filesystem path to install from
    --git <url>             remote repository to clone
    --force                 install regardless of cache slot occupancy
    --disable-ssh           convert SSH repositories to HTTPS for dependencies

Use 'orbit help install' to learn more about the command.
";