use crate::core::uuid::Uuid;
use crate::util::{anyerror::Fault, sha256::Sha256Hash};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use super::{
    pkgid::{PkgId, PkgPart},
    version::{AnyVersion, Version},
};

use crate::core::ip::Ip;

#[derive(Debug)]
pub struct Catalog<'a> {
    inner: HashMap<PkgPart, IpLevel>,
    cache: Option<&'a PathBuf>,
    downloads: Option<&'a PathBuf>,
}

#[derive(Debug, PartialEq)]
pub enum IpState {
    Downloaded,
    Installation,
    Available,
    Unknown,
}

impl std::fmt::Display for IpState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Downloaded => write!(f, "downloaded"),
            Self::Installation => write!(f, "installation"),
            Self::Available => write!(f, "available"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

type IpArchive = ();

#[derive(Debug)]
pub struct IpLevel {
    installs: Vec<Ip>,
    downloads: Vec<Ip>,
    available: Vec<Ip>,
}

impl IpLevel {
    pub fn new() -> Self {
        Self {
            installs: Vec::new(),
            available: Vec::new(),
            downloads: Vec::new(),
        }
    }

    pub fn add_install(&mut self, m: Ip) -> () {
        // only add if not a DST
        if m.is_dynamic() == false {
            self.installs.push(m);
        }
    }

    pub fn add_available(&mut self, m: Ip) -> () {
        self.available.push(m);
    }

    pub fn get_installations(&self) -> &Vec<Ip> {
        &self.installs
    }

    pub fn get_downloads(&self) -> &Vec<Ip> {
        &self.downloads
    }

    pub fn get_availability(&self) -> &Vec<Ip> {
        &self.available
    }

    pub fn is_available(&self) -> bool {
        self.available.is_empty() == false
    }

    pub fn is_installed(&self) -> bool {
        self.installs.is_empty() == false
    }

    pub fn is_downloaded(&self) -> bool {
        self.downloads.is_empty() == false
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_install(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_installations())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_download(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_downloads())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_available(&self, version: &AnyVersion) -> Option<&Ip> {
        Self::get_target_version(version, self.get_availability())
    }

    /// References the ip matching the most compatible version `version`.
    ///
    /// A `dev` version is only searched at the DEV_PATH. Any other version is
    /// first sought for in the cache installations, and if not found then searched
    /// for in the availability space.
    /// Note: `usable` to `false` will not check queued state
    pub fn get(&self, check_downloads: bool, version: &AnyVersion) -> Option<&Ip> {
        match self.get_install(version) {
            Some(ip) => Some(ip),
            None => match check_downloads {
                true => self.get_download(version),
                false => None,
            },
        }
    }

    /// Tracks what level the `manifest` came from.
    pub fn get_state(&self, ip: &Ip) -> IpState {
        if self.installs.iter().find(|f| f == &ip).is_some() {
            IpState::Installation
        } else if self.available.iter().find(|f| f == &ip).is_some() {
            IpState::Available
        } else if self.downloads.iter().find(|f| f == &ip).is_some() {
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
        space
            .iter()
            .filter(|ip| match &target {
                AnyVersion::Specific(v) => {
                    crate::core::version::is_compatible(v, ip.get_man().get_ip().get_version())
                }
                AnyVersion::Latest => true,
            })
            .for_each(|ip| {
                if latest_version.is_none()
                    || ip.get_man().get_ip().get_version()
                        > latest_version
                            .as_ref()
                            .unwrap()
                            .get_man()
                            .get_ip()
                            .get_version()
                {
                    latest_version = Some(ip);
                }
            });
        latest_version
    }
}

impl<'a> Catalog<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            cache: None,
            downloads: None,
        }
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

    /// Searches the `path` for IP downloaded.
    pub fn downloads(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.downloads = Some(&path);
        // @todo: use special method for detecting downloads
        Ok(self)
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
    fn detect(
        mut self,
        path: &PathBuf,
        add: &dyn Fn(&mut IpLevel, Ip) -> (),
        is_pointers: bool,
    ) -> Result<Self, Fault> {
        match is_pointers {
            false => Ip::detect_all(path),
            true => todo!("only detect for available"),
        }?
        .into_iter()
        .for_each(
            |ip| match self.inner.get_mut(&ip.get_man().get_ip().get_name()) {
                Some(lvl) => add(lvl, ip),
                None => {
                    let pkgid = ip.get_man().get_ip().get_name().clone();
                    let mut lvl = IpLevel::new();
                    add(&mut lvl, ip);
                    self.inner.insert(pkgid, lvl);
                    ()
                }
            },
        );
        Ok(self)
    }

    pub fn get_cache_path(&self) -> &PathBuf {
        self.cache.as_ref().unwrap()
    }

    pub fn get_downloads_path(&self) -> &PathBuf {
        self.downloads.as_ref().unwrap()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CacheEntry {
    set: [u8; 1],
    tag: [u8; 1],
    offset: [u8; 14],
}

impl From<&Uuid> for CacheEntry {
    fn from(value: &Uuid) -> Self {
        let bytes: &[u8; 16] = value.get().as_bytes();
        Self {
            set: [bytes[0]; 1],
            tag: [bytes[1]; 1],
            offset: bytes[2..16].try_into().unwrap(),
        }
    }
}

impl CacheEntry {
    /// The first byte in the [Uuid].
    pub fn set(&self) -> String {
        format!("{:02X}", &self.set[0])
    }

    /// The second byte in the [Uuid].
    pub fn tag(&self) -> String {
        format!("{:02X}", &self.tag[0])
    }

    /// The 14 remaining bytes in the [Uuid].
    pub fn offset(&self) -> String {
        self.offset
            .iter()
            .fold(String::new(), |acc, x| acc + &format!("{:02X}", x))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn disp_set() {
        let ce = CacheEntry::from(&Uuid::nil());
        assert_eq!("00", ce.set());
    }

    #[test]
    fn disp_tag() {
        let ce = CacheEntry::from(&Uuid::nil());
        assert_eq!("00", ce.tag());
    }

    #[test]
    fn disp_offset() {
        let ce = CacheEntry::from(&Uuid::nil());
        assert_eq!("0000000000000000000000000000", ce.offset());
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct CacheSlot(String);

impl CacheSlot {
    /// Combines the various components of a cache slot name into a `CacheSlot`.
    pub fn new(name: &PkgPart, version: &Version, checksum: &Sha256Hash) -> Self {
        Self(format!(
            "{}-{}-{}",
            name,
            version,
            checksum.to_string().get(0..10).unwrap()
        ))
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
