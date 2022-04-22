use crate::core::manifest::Manifest;
use std::error::Error;
use crate::core::pkgid::PkgId;
use git2::Repository;

/// An IP is a package that Orbit tracks
pub struct IP {
    path: std::path::PathBuf,
    manifest: Manifest,
}

impl IP {
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

        Ok(Self {
            path: path,
            manifest: Manifest::new(),
        })
    }

    pub fn create_manifest(mut self, pkgid: &PkgId) -> Result<Self, Box<dyn Error>> {
        // create a new manifest
        self.manifest = Manifest::create(self.path.join("Orbit.toml"));
        // fill in fields
        self.manifest.get_mut_doc()["ip"]["name"] = toml_edit::value(pkgid.get_name());
        self.manifest.get_mut_doc()["ip"]["library"] = toml_edit::value(pkgid.get_library().as_ref().unwrap());
        self.manifest.get_mut_doc()["ip"]["vendor"] = toml_edit::value(pkgid.get_vendor().as_ref().unwrap());
        // save the manifest
        self.manifest.save()?;

        // create an empty git repository
        Repository::init(&self.path)?;

        Ok(self)
    }

    pub fn get_path(&self) -> &std::path::PathBuf {
        &self.path
    }
}

#[derive(Debug)]
struct IpError(String);

impl Error for IpError {}

impl std::fmt::Display for IpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}