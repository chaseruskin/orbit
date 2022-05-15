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
        let ip_manifest = crate::commands::edit::find_ip(&self.ip.spec, &manifests)?;
        // get the root path to the manifest
        let mut ip_root = ip_manifest.get_path().clone();
        ip_root.pop();
        // @TODO find the specified version for the specified ip
        let version = "0.0.0";
        let repo = Repository::open(&ip_root)?;

        let tags = repo.tag_names(None)?;
        let mut latest_version: Option<Version> = None;
        for tag in tags.iter().filter_map(|f| {
            if let Some(s) = f {
                match Version::from_str(s) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            } else {
                None
            }
        }) {
            // logic here
        };

        // perform sha256 on the directory after collecting all files
        std::env::set_current_dir(&ip_root)?;
        // must use '.' as current directory when gathering files for consistent checksum
        // @TODO generate cross-platform filepath names (resolve backslash vs. forwardslash)
        let ip_files = crate::core::fileset::gather_current_files(&std::path::PathBuf::from("."));
        let checksum = crate::util::checksum::checksum(&ip_files);
        println!("checksum: {}", checksum);
        // @TODO use luhn algorithm to condense remaining digits in sha256 for directory name

        // use checksum to create new directory slot
        let cache_slot_name = format!("{}-{}-{}", ip_manifest.as_pkgid().get_name(), version, checksum.to_string().get(0..10).unwrap());
        Repository::clone(&ip_root.to_str().unwrap(), &c.get_cache_path().join(&cache_slot_name))?;
        
        std::fs::create_dir(&c.get_cache_path().join(&cache_slot_name))?;
        // @TODO copy contents into cache_slot
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