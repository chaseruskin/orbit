use std::{collections::HashMap, path::PathBuf};
use crate::util::{anyerror::Fault, sha256::Sha256Hash};

use super::{pkgid::{PkgId, PkgPart}, manifest::IpManifest, version::{Version, AnyVersion}, store::Store};

#[derive(Debug)]
pub struct Catalog<'a>(HashMap<PkgId, IpLevel>, Option<Store<'a>>, Option<&'a PathBuf> /*cache path */);

#[derive(Debug)]
pub struct IpLevel {
    dev: Option<IpManifest>,
    installs: Vec<IpManifest>,
    available: Vec<IpManifest>
}

impl IpLevel {
    pub fn new() -> Self {
        Self { dev: None, installs: Vec::new(), available: Vec::new() }
    }

    pub fn add_dev(&mut self, m: IpManifest) -> () {
        self.dev = Some(m);
    }

    pub fn add_install(&mut self, m: IpManifest) -> () {
        self.installs.push(m);
    }

    pub fn add_available(&mut self, m: IpManifest) -> () {
        self.available.push(m);    
    }

    pub fn get_installations(&self) -> &Vec<IpManifest> {
        &self.installs
    }

    pub fn get_availability(&self) -> &Vec<IpManifest> {
        &self.available
    }

    pub fn is_available(&self) -> bool {
        self.available.is_empty() == false
    }

    pub fn is_installed(&self) -> bool {
        self.installs.is_empty() == false
    }

    /// Returns the manifest found on the DEV_PATH.
    pub fn get_dev(&self) -> Option<&IpManifest> {
        self.dev.as_ref()
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_install(&self, version: &AnyVersion) -> Option<&IpManifest> {
        Self::get_target_version(version, self.get_installations())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_available(&self, version: &AnyVersion) -> Option<&IpManifest> {
        Self::get_target_version(version, self.get_availability())
    }

    /// References the ip matching the most compatible version `version`.
    /// 
    /// A `dev` version is only searched at the DEV_PATH. Any other version is
    /// first sought for in the cache installations, and if not found then searched
    /// for in the availability space.
    pub fn get(&self, version: &AnyVersion) -> Option<&IpManifest> {
        match version {
            AnyVersion::Dev => self.get_dev(),
            _ => {
                match self.get_install(version) {
                    Some(ip) => Some(ip),
                    None => self.get_available(version)
                }
            }
        }
    }

    /// Finds the most compatible version matching `target` among the possible `space`.
    /// 
    /// Returns `None` if no compatible version was found.
    /// 
    /// Panics if a development version is entered as `target`.
    fn get_target_version<'a>(target: &AnyVersion, space: &'a Vec<IpManifest>) -> Option<&'a IpManifest> {
        // find the specified version for the given ip
        let mut latest_version: Option<&IpManifest> = None;
        space.iter()
            .filter(|ip| match &target {
                AnyVersion::Specific(v) => crate::core::version::is_compatible(v, ip.get_version()),
                AnyVersion::Latest => true,
                _ => panic!("dev version cannot be filtered")
            })
            .for_each(|ip| {
                if latest_version.is_none() || ip.get_version() > latest_version.as_ref().unwrap().get_version() {
                    latest_version = Some(ip);
                }
            });
        latest_version
    }

    pub fn is_developing(&self) -> bool {
        self.dev.is_some()
    }
}

impl<'a> Catalog<'a> {
    pub fn new() -> Self {
        Self(HashMap::new(), None, None)
    }

    /// Sets the store.
    pub fn store(mut self, path: &'a PathBuf) -> Self {
        self.1 = Some(Store::new(path));
        self
    }

    /// Searches the `path` for IP under development.
    pub fn development(self, path: &PathBuf) -> Result<Self, Fault> {
        self.detect(path, &IpLevel::add_dev)
    }

    /// Uses the cache slot name to check if the directory exists.
    pub fn is_cached_slot(&self, target: &IpManifest) -> bool {
        let _cache_slot = CacheSlot::form(target.get_pkgid().get_name(), target.get_version(), &Sha256Hash::new());
        todo!()
    }

    /// Searches the `path` for IP installed.
    pub fn installations(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.2 = Some(&path);
        self.detect(path, &IpLevel::add_install)
    }

    /// Searches the `path` for IP available.
    pub fn available(self, path: &PathBuf) -> Result<Self, Fault> {
        self.detect(path, &IpLevel::add_available)
    }

    pub fn inner(&self) -> &HashMap<PkgId, IpLevel> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<PkgId, IpLevel> {
        &mut self.0
    }

    /// Returns all possible versions found for the `target` ip.
    /// 
    /// Searches the cache/store, availability space, and development space.
    pub fn get_possible_versions(&self, _: &PkgId) -> Vec<Version> {
        todo!();
    }

    pub fn update_installations(&self) -> () {
        todo!()
    }

    /// Finds all `Orbit.toml` manifest files (markings of an IP) within the provided `path`.
    /// 
    /// This function is generic enough to be used to catch ip at all 3 levels: dev, install, and available.
    fn detect(mut self, path: &PathBuf, add: &dyn Fn(&mut IpLevel, IpManifest) -> ()) -> Result<Self, Fault> {
        crate::core::manifest::IpManifest::detect_all(path)?
            .into_iter()
            .for_each(|ip| {
                match self.0.get_mut(&ip.get_pkgid()) {
                    Some(lvl) => add(lvl, ip),
                    None => { 
                        let pkgid = ip.get_pkgid().clone();
                        let mut lvl = IpLevel::new(); 
                        add(&mut lvl, ip); 
                        self.0.insert(pkgid, lvl); 
                        ()
                    },
                }
            });
        Ok(self)
    }

    pub fn get_store(&self) -> &Store {
        self.1.as_ref().unwrap()
    }

    pub fn get_cache_path(&self) -> &PathBuf {
        self.2.as_ref().unwrap()
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct CacheSlot(String);

impl CacheSlot {
    pub fn new() -> Self {
        Self(String::new())
    }

    /// Combines the various components of a cache slot name into a `CacheSlot`.
    pub fn form(name: &PkgPart, version: &Version, checksum: &Sha256Hash) -> Self {
        Self(format!("{}-{}-{}", name, version, checksum.to_string().get(0..10).unwrap()))
    }
}

impl AsRef<str> for CacheSlot {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}