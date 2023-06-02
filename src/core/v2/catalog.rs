use std::{collections::{HashMap, HashSet}, path::PathBuf};
use crate::util::{anyerror::Fault, sha256::Sha256Hash};

use super::super::{pkgid::{PkgId, PkgPart}, manifest::IpManifest, version::{Version, AnyVersion}, store::Store, vendor::VendorManifest};

use crate::core::v2::ip::Ip;

#[derive(Debug)]
pub struct Catalog<'a> {
    inner: HashMap<PkgPart, IpLevel>, 
    store: Option<Store<'a>>, 
    cache: Option<&'a PathBuf>,
    queue: Option<&'a PathBuf>,
}

#[derive(Debug, PartialEq)]
pub enum IpState {
    Downloaded,
    Installation,
    Available,
    Unknown
}

impl std::fmt::Display for IpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Downloaded => write!(f, "downloaded"),
            Self::Installation => write!(f, "installation"),
            Self::Available => write!(f, "available"),
            Self::Unknown => write!(f, "unknown")
        }
    }
}

#[derive(Debug)]
pub struct IpLevel {
    dev: Option<Ip>,
    installs: Vec<Ip>,
    queue: Vec<Ip>,
    available: Vec<Ip>
}

impl IpLevel {
    pub fn new() -> Self {
        Self { dev: None, installs: Vec::new(), available: Vec::new(), queue: Vec::new(), }
    }

    pub fn is_available_or_in_store(&self, store: &Store, pkgid: &PkgId, v: &AnyVersion) -> bool {
        self.get_available(v).is_some() || IpManifest::from_store(&store, &pkgid, v).unwrap_or(None).is_some()
    }

    pub fn add_dev(&mut self, m: Ip) -> () {
        self.dev = Some(m);
    }

    pub fn add_install(&mut self, m: Ip) -> () {
        // only add if not a DST
        if m.is_dynamic() == false {
            self.installs.push(m);
        }
    }

    pub fn add_queue(&mut self, m: Ip) -> () {
        self.queue.push(m);
    }

    pub fn add_available(&mut self, m: Ip) -> () {
        self.available.push(m);    
    }

    pub fn get_installations(&self) -> &Vec<Ip> {
        &self.installs
    }

    pub fn get_availability(&self) -> &Vec<Ip> {
        &self.available
    }

    pub fn get_queued(&self) -> &Vec<Ip> {
        &self.queue
    }

    pub fn is_available(&self) -> bool {
        self.available.is_empty() == false
    }

    pub fn is_installed(&self) -> bool {
        self.installs.is_empty() == false
    }

    pub fn is_queued(&self) -> bool {
        self.queue.is_empty() == false
    }

    /// Returns the manifest found on the DEV_PATH.
    pub fn get_dev(&self) -> Option<&Ip> {
        self.dev.as_ref()
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_install(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_installations())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_available(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_availability())
    }

    pub fn get_queue(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_queued())
    }

    /// References the ip matching the most compatible version `version`.
    /// 
    /// A `dev` version is only searched at the DEV_PATH. Any other version is
    /// first sought for in the cache installations, and if not found then searched
    /// for in the availability space.
    /// Note: `usable` to `false` will not check queued state
    pub fn get(&self, check_queue: bool, version: &AnyVersion) -> Option<&Ip> {
        match self.get_install(version) {
            Some(ip) => Some(ip),
            None => {
                match check_queue {
                    true => self.get_queue(version),
                    false => None,
                }
            },
        }
    }

    /// Tracks what level the `manifest` came from.
    pub fn get_state(&self, ip: &Ip) -> IpState {
        if self.installs.iter().find(|f| f == &ip).is_some() { 
            IpState::Installation 
        } else if self.available.iter().find(|f| f == &ip).is_some() {
            IpState::Available
        } else if self.queue.iter().find(|f| f == &ip).is_some() {
            IpState::Downloaded
        } else {
            IpState::Unknown
        } 
    }

    /// Finds the most compatible version matching `target` among the possible `space`.
    /// 
    /// Returns `None` if no compatible version was found.
    /// 
    /// Panics if a development version is entered as `target`.
    fn get_target_version<'a>(target: &AnyVersion, space: &'a Vec<Ip>) -> Option<&'a Ip> {
        // find the specified version for the given ip
        let mut latest_version: Option<&Ip> = None;
        space.iter()
            .filter(|ip| match &target {
                AnyVersion::Specific(v) => crate::core::version::is_compatible(v, ip.get_man().get_ip().get_version()),
                AnyVersion::Latest => true,
            })
            .for_each(|ip| {
                if latest_version.is_none() || ip.get_man().get_ip().get_version() > latest_version.as_ref().unwrap().get_man().get_ip().get_version() {
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
        Self {
            inner: HashMap::new(), 
            store: None, 
            cache: None,
            queue: None,
        }
    }

    /// Sets the store.
    pub fn store(mut self, path: &'a PathBuf) -> Self {
        self.store = Some(Store::new(path));
        self
    }

    /// Searches the `path` for IP under development.
    pub fn development(self, path: &PathBuf) -> Result<Self, Fault> {
        self.detect(path, &IpLevel::add_dev, false)
    }

    /// Uses the cache slot name to check if the directory exists.
    pub fn is_cached_slot(&self, slot: &CacheSlot) -> bool {
        self.get_cache_path().join(slot.as_ref()).is_dir()
    }

    /// Searches the `path` for IP installed.
    pub fn installations(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.cache = Some(&path);
        self.detect(path, &IpLevel::add_install, false)
    }

    /// Searches the `path` for IP in the queue.
    pub fn queue(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.queue = Some(&path);
        self.detect(path, &IpLevel::add_queue, false)
    }

    /// Searches the `path` for IP available.
    pub fn available(self, vendors: &HashMap<PkgPart, VendorManifest>) -> Result<Self, Fault> {
        let mut catalog = self;
        for (_, v) in vendors {
            catalog = catalog.detect(&v.get_root(), &IpLevel::add_available, true)?;
        }
        Ok(catalog)
    }

    pub fn inner(&self) -> &HashMap<PkgPart, IpLevel> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<PkgPart, IpLevel> {
        &mut self.inner
    }

    /// Returns all possible versions found for the `target` ip.
    /// 
    /// Returns `None` if the id is not found in the catalog.
    pub fn get_possible_versions(&self, id: &PkgPart) -> Option<Vec<&Version>> {
        let kaban = self.inner.get(&id)?;
        let mut set = HashSet::new();
        for ip in kaban.get_installations() {
            set.insert(ip.get_man().get_ip().get_version());
        }
        let mut arr: Vec<&Version> = set.into_iter().collect();
        arr.sort();
        arr.reverse();
        Some(arr)
    }

    pub fn update_installations(&mut self) -> () {
        todo!()
    }

    /// Finds all `Orbit.toml` manifest files (markings of an IP) within the provided `path`.
    /// 
    /// This function is generic enough to be used to catch ip at all 3 levels: dev, install, and available.
    fn detect(mut self, path: &PathBuf, add: &dyn Fn(&mut IpLevel, Ip) -> (), is_pointers: bool) -> Result<Self, Fault> {
        match is_pointers {
            false => Ip::detect_all(path),
            true => todo!("only detect for available")
        }?.into_iter()
            .for_each(|ip| {
                match self.inner.get_mut(&ip.get_man().get_ip().get_name()) {
                    Some(lvl) => add(lvl, ip),
                    None => { 
                        let pkgid = ip.get_man().get_ip().get_name().clone();
                        let mut lvl = IpLevel::new(); 
                        add(&mut lvl, ip); 
                        self.inner.insert(pkgid, lvl); 
                        ()
                    },
                }
            });
        Ok(self)
    }

    pub fn get_store(&self) -> &Store {
        self.store.as_ref().unwrap()
    }

    pub fn get_cache_path(&self) -> &PathBuf {
        self.cache.as_ref().unwrap()
    }

    pub fn get_queue_path(&self) -> &PathBuf {
        self.queue.as_ref().unwrap()
    }
}


#[derive(PartialEq, Debug, Clone)]
pub struct CacheSlot(String);

impl CacheSlot {
    /// Combines the various components of a cache slot name into a `CacheSlot`.
    pub fn new(name: &PkgPart, version: &Version, checksum: &Sha256Hash) -> Self {
        Self(format!("{}-{}-{}", name, version, checksum.to_string().get(0..10).unwrap()))
    }
}

impl AsRef<str> for CacheSlot {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub enum CatalogError {
    SuggestInstall(PkgId, AnyVersion),
    NoVersionForIp(PkgId, AnyVersion),
}

impl std::error::Error for CatalogError {}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SuggestInstall(target, version) => write!(f, "ip '{}' is not installed but is available\n\nTry installing the ip: `orbit install --ip {} -v {}`", target, target, version),
            Self::NoVersionForIp(pkgid, version) => write!(f, "ip '{}' has no version '{}'", pkgid, version),
        }
    }
}