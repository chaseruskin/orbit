use std::{collections::HashMap, path::PathBuf};
use crate::util::anyerror::Fault;

use super::{pkgid::PkgId, manifest::IpManifest, version::Version};

#[derive(Debug)]
pub struct Catalog(HashMap<PkgId, IpLevel>);


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

    pub fn get_dev(&self) -> Option<&IpManifest> {
        self.dev.as_ref()
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

    pub fn is_developing(&self) -> bool {
        self.dev.is_some()
    }
}

impl Catalog {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Searches the `path` for IP under development.
    pub fn development(self, path: &PathBuf) -> Result<Self, Fault> {
        self.detect(path, &IpLevel::add_dev)
    }

    /// Searches the `path` for IP installed.
    pub fn installations(self, path: &PathBuf) -> Result<Self, Fault> {
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

    /// Find a matching manifest for the requested `pkgid` and `version`.
    /// 
    /// Looks in the cache/store and the availability space.
    fn find() {
        todo!();
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
}