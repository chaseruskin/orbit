use std::path::PathBuf;

use crate::util::{anyerror::Fault, filesystem};
use super::{pkgid::PkgId, manifest::IpManifest};

#[derive(Debug, PartialEq)]
pub struct Store<'a> {
    root: &'a PathBuf
}

impl<'a> Store<'a> {
    pub fn new(root: &'a PathBuf) -> Self {
        Store { root: root }
    }

    /// Stashes the `repo` for the `ip` into the .orbit/store folder.
    /// 
    /// It will completely replace the existing store slot or create a new one.
    /// Assumes the `ip` is not located within the store.
    pub fn store(&self, ip: &IpManifest) -> Result<PathBuf, Fault> {
        let id_dir = ip.get_pkgid().into_hash().to_string();
        let store_ip_dir = self.root.join(&id_dir);
        // force removal of the existing directory
        if store_ip_dir.exists() == true {
            std::fs::remove_dir_all(&store_ip_dir)?;
        }
        // copy the repository to the store location
        filesystem::copy(&ip.get_root(), &store_ip_dir, false)?;
        Ok(store_ip_dir)
    }

    /// Tries to access the current directory already placed in the Orbit store.
    pub fn as_stored(&self, ip: &PkgId) -> Option<PathBuf> {
        let store_ip_dir = self.root.join(ip.into_hash().to_string());
        match std::path::Path::exists(&store_ip_dir) {
            true => Some(store_ip_dir),
            false => None,
        }
    }

    /// Checks if the ip is already stored.
    pub fn is_stored(&self, ip: &PkgId) -> bool {
        std::path::Path::exists(&self.root.join(ip.into_hash().to_string()))
    }
}