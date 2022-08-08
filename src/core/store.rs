use std::{path::PathBuf, collections::hash_map::DefaultHasher};

use crate::util::{anyerror::Fault, filesystem, url::Url};
use super::{pkgid::PkgId, manifest::IpManifest};

#[derive(Debug, PartialEq)]
pub struct Store<'a> {
    root: &'a PathBuf
}

// @todo: use this for storage identification? harder to look up though given must find a 
// repository, but can be more robust against naming identifier conflicts
#[derive(Debug, PartialEq, Hash, Eq)]
struct StoreId<'a> {
    pkgid: &'a PkgId,
    remote: Option<&'a Url>,
}

use std::hash::Hash;
use std::hash::Hasher;

impl<'a> StoreId<'a> {
    fn _from_ip(ip: &'a IpManifest) -> Self {
        StoreId { pkgid: ip.get_pkgid(), remote: ip.get_repository() }
    }

    fn _into_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
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

    /// Checks if the provided `path` is within the store.
    pub fn is_path_in_store(&self,  path: &std::path::PathBuf) -> bool {
        crate::util::filesystem::remove_base(self.root, path).parent().unwrap().parent().is_none()
    }

    /// Checks if the ip is already stored.
    pub fn is_stored(&self, ip: &PkgId) -> bool {
        std::path::Path::exists(&self.root.join(ip.into_hash().to_string()))
    }
}