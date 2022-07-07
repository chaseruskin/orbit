use std::path::PathBuf;
use crate::util::anyerror::Fault;
use super::{ip::Ip, pkgid::PkgId};
use git2::Repository;

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
    pub fn store(&self, ip: &Ip) -> Result<PathBuf, Fault> {
        let id_dir = ip.get_manifest().get_pkgid().into_hash().to_string();
        let store_ip_dir = self.root.join(&id_dir);
        // force removal of the existing directory
        if store_ip_dir.exists() == true {
            std::fs::remove_dir_all(&store_ip_dir)?;
        }
        // create new directory to store
        std::fs::create_dir(&store_ip_dir)?;
        // clone the repository to the store location
        Repository::clone(&ip.get_path().to_str().unwrap(), &store_ip_dir)?;
        Ok(store_ip_dir)
    }

    /// Tries to access the current ip already placed in the Orbit store.
    pub fn as_stored(&self, ip: &PkgId) -> Result<Option<Ip>, Fault> {
        let store_ip_dir = self.root.join(ip.into_hash().to_string());
        if std::path::Path::exists(&store_ip_dir) == false {
            Ok(None)
        } else {
            // grab the ip manifest there
            Ok(Some(Ip::init_from_path(store_ip_dir)?))
        }
    }

    /// Checks if the ip is already stored.
    pub fn is_stored(&self, ip: &PkgId) -> bool {
        std::path::Path::exists(&self.root.join(ip.into_hash().to_string()))
    }
}