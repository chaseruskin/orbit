use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::Positional; // Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::core::version::Version;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgIdError;
use crate::core::version::VersionError;

#[derive(Debug, PartialEq)]
struct IpSpecVersion {
    spec: PkgId,
    version: InstallVersion,
}

#[derive(Debug, PartialEq)]
enum InstallVersion {
    Latest,
    Specific(Version),
}

impl std::str::FromStr for InstallVersion {
    type Err = VersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if crate::util::strcmp::cmp_ascii_ignore_case(s, "latest") {
            Ok(Self::Latest)
        } else {
            Ok(Self::Specific(Version::from_str(s)?))
        }
    }
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
                version: InstallVersion::from_str(ver_str)?,
            })
        // if did not find a '@' symbol, default to latest
        } else {
            Ok(Self {
                spec: PkgId::from_str(s)?,
                version: InstallVersion::Latest,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: IpSpecVersion,
}

impl FromCli for Install {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Install {
            ip: cli.require_positional(Positional::new("ip[@version]"))?,
        });
        command
    }
}

use git2::Repository;
use std::str::FromStr;

impl Command for Install {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // gather all manifests
        let manifests = crate::core::manifest::find_dev_manifests(c.get_development_path().as_ref().unwrap())?;
        let ip_manifest = crate::core::ip::find_ip(&self.ip.spec, &manifests)?;
        // get the root path to the manifest
        let mut ip_root = ip_manifest.get_path().clone();
        ip_root.pop();

        // gather all version tags matching the version given on command-line
        let tags = {
            // this ver_str is needed to keep lifetime of &str for pattern
            #[allow(unused_assignments)] 
            let mut ver_str = String::new();
            let version_pattern: Option<&str> = match &self.ip.version {
                InstallVersion::Specific(v) => {
                    ver_str = v.to_string();
                    Some(&ver_str)
                },
                InstallVersion::Latest => None,
            };
            let repo = Repository::open(&ip_root)?;
            // find the highest fitting version
            repo.tag_names(version_pattern)?
        };

        // find the specified version for the given ip
        let mut latest_version: Option<Version> = None;
        tags.into_iter()
            .filter_map(|f| {
                if let Some(s) = f {
                    match Version::from_str(s) {
                        Ok(v) => Some(v),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .for_each(|tag| {
                if latest_version.is_none() || &tag > latest_version.as_ref().unwrap() {
                    latest_version = Some(tag);
                }
            });

        if let Some(ver) = &latest_version {
            println!("detected version {}", ver) 
        } else {
            panic!("no verison found for {:?}", self.ip.version);
        }
        let version = latest_version.unwrap();

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
        let checksum = crate::util::checksum::checksum(&ip_files);
        println!("checksum: {}", checksum);
        // @TODO use luhn algorithm to condense remaining digits in sha256 for directory name

        // use checksum to create new directory slot
        let cache_slot_name = format!("{}-{}-{}", ip_manifest.as_pkgid().get_name(), version, checksum.to_string().get(0..10).unwrap());
        let cache_slot = c.get_cache_path().join(&cache_slot_name);
        std::fs::create_dir(&cache_slot)?;
        // move contents into cache slot
        std::fs::rename(temp, &cache_slot)?;

        self.run()
    }
}

impl Install {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        todo!()
    }
}

const HELP: &str = "\
Quick help sentence about command.

Usage:
    orbit install [options] <ip[@version]>

Args:
    <ip[@version]>    ip spec along with optional version tag

Options:
    N/A

Use 'orbit help template' to learn more about the command.
";