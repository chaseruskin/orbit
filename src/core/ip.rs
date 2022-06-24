use crate::core::manifest::IpManifest;
use std::error::Error;
use crate::core::pkgid::PkgId;
use git2::Repository;

/// An IP is a package that Orbit tracks
pub struct Ip {
    /// the root directory of the project (Orbit.toml location)
    path: std::path::PathBuf,
    manifest: IpManifest,
}

impl Ip {
    /// Creates an `Ip` from a manifest.
    pub fn from_manifest(manifest: IpManifest) -> Self {
        Self { 
            path: manifest.0.get_path().as_path().parent().unwrap().to_path_buf(), 
            manifest: manifest 
        }
    }


    /// Creates an `Ip` located at `path` as its root.
    pub fn from_path(path: std::path::PathBuf) -> Self {
        Self {
            path: path,
            manifest: IpManifest::new(),
        }
    }

    pub fn into_manifest(self) -> IpManifest {
        self.manifest
    }

    /// Gathers the list of primary design units for the current ip.
    pub fn collect_units(&self) -> Vec<PrimaryUnit> {
        // collect all files
        let files = crate::core::fileset::gather_current_files(&self.path);
        primaryunit::collect_units(&files)
    }

    /// Creates a new IP at the `path`
    /// 
    /// A manifest is created one level within `path`.
    pub fn new(path: std::path::PathBuf, force: bool) -> Result<Self, Box<dyn Error>> {
        if std::path::Path::exists(&path) == true {
            // remove the entire existing directory
            if force == true {
                std::fs::remove_dir_all(&path)?;
            // error if directories exist
            } else {
                return Err(Box::new(IpError(format!("failed to create new ip because directory '{}' already exists", path.display()))))
            }
        }
        // create all directories if the do not exist
        std::fs::create_dir_all(&path)?;

        // @TODO issue warning if the path it was placed is outside of DEV_PATH or if DEV_PATH is not set

        Ok(Self {
            path: path,
            manifest: IpManifest::new(),
        })
    }

    /// Creates a new manifest and writes it to disk at the `path`.
    /// 
    /// Assumes the `pkgid` is fully qualified.
    pub fn create_manifest(mut self, pkgid: &PkgId) -> Result<Self, Box<dyn Error>> {
        // initialize a new manifest
        self.manifest = IpManifest::init(self.path.join(manifest::IP_MANIFEST_FILE));
        // fill in fields
        self.manifest.0.write("ip", "name", pkgid.get_name());
        self.manifest.0.write("ip", "library", pkgid.get_library().as_ref().unwrap());
        self.manifest.0.write("ip", "vendor", pkgid.get_vendor().as_ref().unwrap());
        // save the manifest
        self.manifest.0.save()?;

        // create an empty git repository
        Repository::init(&self.path)?;

        Ok(self)
    }

    pub fn get_path(&self) -> &std::path::PathBuf {
        &self.path
    }
}

use crate::util::overdetsys;
use crate::core::manifest;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::AnyError;

use super::vhdl::primaryunit::{PrimaryUnit, self};

/// Given a partial/full ip specification `ip_spec`, sift through the manifests
/// for a possible determined unique solution.
pub fn find_ip<'a, 'b>(ip_spec: &'b PkgId, universe: Vec<&'a PkgId>) -> Result<PkgId, AnyError> {
    // try to find ip name
    let space: Vec<Vec<PkgPart>> = universe.iter().map(|f| { f.into_full_vec().unwrap() }).collect();
    let result = match overdetsys::solve(space, ip_spec.iter()) {
        Ok(r) => r,
        Err(e) => match e {
            overdetsys::OverDetSysError::NoSolution => Err(AnyError(format!("no ip as '{}' exists", ip_spec)))?,
            overdetsys::OverDetSysError::Ambiguous(set) => {
                // assemble error message
                let mut set = set.into_iter().map(|f| PkgId::from_vec(f) );
                let mut content = String::new();
                while let Some(s) = set.next() {
                    content.push_str(&format!("    {}\n", s.to_string()));
                }
                Err(AnyError(format!("ambiguous ip '{}' yields multiple solutions:\n{}", ip_spec, content)))?
            }
        }
    };

    let full_ip = PkgId::from_vec(result);
    // find the full ip name among the manifests to get the path
    Ok(universe.iter().find(|&&f| { full_ip == f }).unwrap().clone())
}

#[derive(Debug)]
struct IpError(String);

impl Error for IpError {}

impl std::fmt::Display for IpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}