use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::core::version::Version;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgIdError;
use crate::core::version::{VersionError, AnyVersion};
use crate::util::sha256;
use crate::util::sha256::Sha256Hash;

#[derive(Debug, PartialEq)]
struct IpSpecVersion {
    spec: PkgId,
    version: AnyVersion,
}

impl From<PkgIdError> for AnyError {
    fn from(e: PkgIdError) -> Self { 
        AnyError(e.to_string())
    }
}

impl From<VersionError> for AnyError {
    fn from(e: VersionError) -> Self { 
        AnyError(e.to_string()) 
    }
}

impl std::str::FromStr for IpSpecVersion {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((pkgid_str, ver_str)) = s.rsplit_once('@') {
            Ok(Self {
                spec: PkgId::from_str(pkgid_str)?,
                version: AnyVersion::from_str(ver_str)?,
            })
        // if did not find a '@' symbol, default to latest
        } else {
            Ok(Self {
                spec: PkgId::from_str(s)?,
                version: AnyVersion::Latest,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: PkgId,
    version: Option<AnyVersion>,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Install {
            version: cli.check_option(Optional::new("ver").switch('v'))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        });
        command
    }
}

use git2::Repository;
use std::str::FromStr;
use crate::commands::search::Search;

impl Command for Install {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // verify user is not requesting the dev version to be installed
        let version = match &self.version {
            Some(v) => {
                if v == &AnyVersion::Dev {
                    return Err(AnyError(format!("{}", "a dev version cannot be installed to the cache")))?
                } else {
                    v
                }
            },
            None => &AnyVersion::Latest
        };

        // @TODO gather all manifests from all 3 levels
        let universe = Search::all_pkgid((c.get_development_path().unwrap(), c.get_cache_path(), &c.get_vendor_path()))?;
        let target = crate::core::ip::find_ip(&self.ip, universe.keys().into_iter().collect())?;
        // @TODO gather all possible versions found for this IP
        let ip_manifest = &universe.get(&target).as_ref().unwrap().0.as_ref().unwrap();

        // get the root path to the manifest
        let mut ip_root = ip_manifest.0.get_path().clone();
        ip_root.pop();

        let repo = Repository::open(&ip_root)?;
        // find the specified version for the given ip
        let space = gather_version_tags(&repo)?;
        let version = get_target_version(&version, &space, &target)?;
        println!("detected version {}", version);

        // move into temporary directory to compute checksum for the tagged version
        let temp = tempfile::tempdir()?;
        let repo = Repository::clone(&ip_root.to_str().unwrap(), &temp)?;
        // get the tag
        let obj = repo.revparse_single(version.to_string().as_ref())?;
        // checkout code at the tag's marked timestamp
        repo.checkout_tree(&obj, None)?;

        // perform sha256 on the directory after collecting all files
        std::env::set_current_dir(&temp)?;
        // must use '.' as current directory when gathering files for consistent checksum
        let ip_files = crate::core::fileset::gather_current_files(&std::path::PathBuf::from("."));
        // println!("{:?}", ip_files);
        let checksum = crate::util::checksum::checksum(&ip_files);
        println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = format!("{}-{}-{}", ip_manifest.as_pkgid().get_name(), version, checksum.to_string().get(0..10).unwrap());
        let cache_slot = c.get_cache_path().join(&cache_slot_name);
        if std::path::Path::exists(&cache_slot) == true {
            // verify the installed version is valid
            if let Some(sha) = Self::get_checksum_proof(&cache_slot, 0) {
                if sha == checksum {
                    return Err(AnyError(format!("IP {} version {} is already installed", ip_manifest.as_pkgid(), version)))?
                }
            }
            println!("info: reinstalling due to bad checksum");
            // blow directory up for re-install
            std::fs::remove_dir_all(&cache_slot)?;
        }
        std::fs::create_dir(&cache_slot)?;
        // copy contents into cache slot
        let options = fs_extra::dir::CopyOptions::new();
        let mut from_paths = Vec::new();
        for dir_entry in std::fs::read_dir(temp.path())? {
            match dir_entry {
                Ok(d) => from_paths.push(d.path()),
                Err(_) => (),
            }
        }
        // copy rather than rename because of windows issues
        fs_extra::copy_items(&from_paths, &cache_slot, &options)?;
        // write the checksum to the directory
        std::fs::write(&cache_slot.join(crate::core::fileset::ORBIT_SUM_FILE), checksum.to_string().as_bytes())?;
        self.run()
    }
}

/// Finds the most compatible version matching `ver` among the possible `space`.
/// 
/// Errors if no version was found.
pub fn get_target_version<'a>(ver: &AnyVersion, space: &'a Vec<Version>, target: &PkgId) -> Result<&'a Version, AnyError> {
    // find the specified version for the given ip
    let mut latest_version: Option<&Version> = None;
    space.into_iter()
    .filter(|f| match &ver {
        AnyVersion::Specific(v) => crate::core::version::is_compatible(v, f),
        AnyVersion::Latest => true,
        _ => panic!("dev version cannot be filtered")
    })
    .for_each(|tag| {
        if latest_version.is_none() || &tag > latest_version.as_ref().unwrap() {
            latest_version = Some(tag);
        }
    });
    match latest_version {
        Some(v) => Ok(v),
        None => Err(AnyError(format!("\
ip '{}' has no version available as {}

To see all versions try `orbit probe {} --tags`", target, ver, target))),
    }
}

/// Collects all version tags from the given `repo` repository.
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
    /// Gets the already calculated checksum from an installed IP from '.orbit-checksum'.
    /// 
    /// This fn can return the different levels of the check-sum, whether its the dynamic
    /// SHA (level 1) or the original SHA (level 0).
    /// 
    /// Returns `None` if the file does not exist, is unable to read into a string, or
    /// if the sha cannot be parsed.
    fn get_checksum_proof(p: &std::path::PathBuf, level: u8) -> Option<Sha256Hash> {
        let sum_file = p.join(crate::core::fileset::ORBIT_SUM_FILE);
        if std::path::Path::exists(&sum_file) == false {
            None
        } else {
            match std::fs::read_to_string(&sum_file) {
                Ok(text) => {
                    let mut sums = text.split_terminator('\n').skip(level.into());
                    match sha256::Sha256Hash::from_str(&sums.next().expect("level was out of bounds")) {
                        Ok(sha) => Some(sha),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // todo!()
        Ok(())
    }
}

const HELP: &str = "\
Places an immutable version of an ip to the cache for dependency usage.

Usage:
    orbit install [options] <ip>

Args:
    <ip>    ip spec along with optional version tag

Options:
    --ver, -v <version>     version to install
    --force                 install regardless of cache slot occupancy

Use 'orbit help install' to learn more about the command.
";