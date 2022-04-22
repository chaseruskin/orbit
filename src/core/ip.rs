use crate::core::manifest::Manifest;
use std::error::Error;
use crate::core::pkgid::PkgId;
use git2::Repository;

/// An IP is a package that Orbit tracks
pub struct IP {
    path: std::path::PathBuf,
    manifest: Manifest
}

impl IP {
    pub fn new(path: std::path::PathBuf, pkgid: &PkgId, force: bool) -> Result<Self, Box<dyn Error>> {
        if std::path::Path::exists(&path) == true {
            // remove the entire existing directory
            if force == true {
                std::fs::remove_dir_all(&path)?;
            } else {
                return Err(Box::new(IpError(format!("failed to create new ip because directory '{}' already exists", path.display()))))
            }
        }
        // create all directories if the do not exist
        std::fs::create_dir_all(&path)?;
        // error if directories exist
        // create a new manifest
        let mut manifest = Manifest::new(path.join("Orbit.toml"));
        // fill in fields
        manifest.get_mut_doc()["ip"]["name"] = toml_edit::value(pkgid.get_name());
        manifest.get_mut_doc()["ip"]["library"] = toml_edit::value(pkgid.get_library().as_ref().unwrap());
        manifest.get_mut_doc()["ip"]["vendor"] = toml_edit::value(pkgid.get_vendor().as_ref().unwrap());
        // save the manifest
        manifest.save()?;

        // create an empty git repository
        Repository::init(&path)?;

        Ok(Self {
            path: path,
            manifest: manifest
        })
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