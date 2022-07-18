use crate::Command;
use crate::FromCli;
use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::manifest;
use crate::core::manifest::IpManifest;
use crate::core::version;
use crate::interface::cli::Cli;
use crate::interface::arg::Optional;
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::core::version::Version;
use crate::util::anyerror::{AnyError, Fault};
use crate::core::version::AnyVersion;

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: Option<PkgId>,
    path: Option<std::path::PathBuf>,
    git: Option<String>,
    version: AnyVersion,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Install {
            git: cli.check_option(Optional::new("git").value("url"))?,
            path: cli.check_option(Optional::new("path"))?,
            version: cli.check_option(Optional::new("ver").switch('v'))?.unwrap_or(AnyVersion::Latest),
            ip: cli.check_option(Optional::new("ip"))?,
        });
        command
    }
}

use colored::Colorize;
use git2::Repository;
use tempfile::tempdir;
use crate::core::store::Store;
use std::path::PathBuf;
use std::str::FromStr;
use crate::core::extgit::ExtGit;

impl Command for Install {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // verify user is not requesting the dev version to be installed
        match &self.version {
            AnyVersion::Dev => return Err(AnyError(format!("{}", "a dev version cannot be installed to the cache")))?,
            _ => ()
        };
        // let temporary directory exist for lifetime of install in case of using it
        let tempdir = tempdir()?;

        let store = Store::new(c.get_store_path());

        // get to the repository (root path)
        let ip_root = if let Some(ip) = &self.ip {
            // gather the catalog (all manifests)
            let mut catalog = Catalog::new()
                .store(c.get_store_path())
                .development(c.get_development_path().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?;
            let ids = catalog.inner().keys().map(|f| { f }).collect();

            let target = crate::core::ip::find_ip(ip, ids)?;
            // gather all possible versions found for this IP
            let status = catalog.inner_mut().remove(&target).take().unwrap();

            // check the store/ for the repository
            if let Some(ip) = store.as_stored(&target)? {
                ip.get_root()
            // clone from remote repository if exists
            } else if let Some(url) = status.try_repository() {
                let path = tempdir.path().to_path_buf();
                println!("info: fetching repository ...");
                // @TODO add --disable ssh here
                ExtGit::new(None).clone(&url.to_string(), &path)?;
                path
            } else {
                // @TODO last resort, clone the actual dev directory to a temp folder
                panic!("no repository to access ip")
            }
        } else if let Some(url) = &self.git {
            // clone from remote repository
            let path = tempdir.path().to_path_buf();
            println!("info: fetching repository ...");
            ExtGit::new(None).clone(url, &path)?;
            path
        } else if let Some(path) = &self.path {
            // traverse filesystem
            path.clone()
        } else {
            return Err(AnyError(format!("select an option to install from '{}', '{}', or '{}'", "--ip".yellow(), "--git".yellow(), "--path".yellow())))?
        };
        // enter action
        self.run(&ip_root, c.get_cache_path(), c.force, store)
    }
}

/// Collects all version git tags from the given `repo` repository.
/// 
/// The tags must follow semver `[0-9]*.[0-9]*.[0-9]*` specification.
fn gather_version_tags(repo: &Repository) -> Result<Vec<Version>, Box<dyn std::error::Error>> {
    let tags = repo.tag_names(Some("*.*.*"))?;
    Ok(tags.into_iter()
        .filter_map(|f| {
            match Version::from_str(f?) {
                Ok(v) => Some(v),
                Err(_) => None,
            }
        })
        .collect())
}

impl Install {
    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    /// 
    /// Errors if the ip is already installed unless `force` is true.
    pub fn install(installation_path: &PathBuf, version: &AnyVersion, cache_root: &std::path::PathBuf, force: bool, store: &Store) -> Result<IpManifest, Fault> {
        let repo = Repository::open(&installation_path)?;

        // find the specified version for the given ip
        let space = gather_version_tags(&repo)?;
        let version_space: Vec<&Version> = space.iter().collect();
        let version = version::get_target_version(&version, &version_space)?;

        println!("detected version {}", version);
        ExtGit::checkout_tag_state(&repo, &version)?;

        // make an ip manifest
        let ip = IpManifest::from_path(installation_path)?;
        let target = ip.get_pkgid();

        // move into stored directory to compute checksum for the tagged version
        let temp = match store.is_stored(&target) {
            true => ip.get_root(),
            // throw repository into the store/ for future use
            false => store.store(&ip)?,
        };

        let repo = Repository::open(&temp)?;
        ExtGit::checkout_tag_state(&repo, &version)?;
    
        // perform sha256 on the directory after collecting all files
        std::env::set_current_dir(&temp)?;

        // must use '.' as current directory when gathering files for consistent checksum
        let ip_files = crate::util::filesystem::gather_current_files(&std::path::PathBuf::from("."));
        
        let checksum = crate::util::checksum::checksum(&ip_files);
        println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = CacheSlot::form(target.get_name(), &version, &checksum);
        let cache_slot = cache_root.join(&cache_slot_name.as_ref());
        if std::path::Path::exists(&cache_slot) == true {
            // check if we should proceed with force regardless if the installation is valid
            if force == true {
                std::fs::remove_dir_all(&cache_slot)?;
            } else {
                let cached_ip = IpManifest::from_path(&cache_slot)?;
                // verify the installed version is valid
                if let Some(sha) = cached_ip.get_checksum_proof(0) {
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
        std::fs::create_dir(&cache_slot)?;
        // copy contents into cache slot
        let options = fs_extra::dir::CopyOptions::new();
        let mut from_paths = Vec::new();
        for dir_entry in std::fs::read_dir(temp)? {
            match dir_entry {
                Ok(d) => if d.file_name() != ".git" || d.file_type()?.is_dir() != true { from_paths.push(d.path()) },
                Err(_) => (),
            }
        }
        // note: copy rather than rename because of windows issues
        fs_extra::copy_items(&from_paths, &cache_slot, &options)?;
        // write the checksum to the directory
        std::fs::write(&cache_slot.join(manifest::ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        Ok(IpManifest::from_path(&cache_slot)?)
    }

    fn run(&self, installation_path: &PathBuf, cache_root: &std::path::PathBuf, force: bool, store: Store) -> Result<(), Fault> {
        let _ = Self::install(&installation_path, &self.version, &cache_root, force, &store)?;
        // @TODO auto-install dependencies of the current ip as well
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

Use 'orbit help install' to learn more about the command.
";