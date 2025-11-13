//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use crate::core::uuid::Uuid;
use crate::error::{Error, Hint};
use crate::util::anyerror::CodeFault;
use crate::util::{anyerror::Fault, sha256::Sha256Hash};
use std::fmt::Display;
use std::fs::read_dir;
use std::str::FromStr;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use super::channel::Channel;
use super::project_archive::ARCHIVE_EXT;
use super::project_pointer::ProjectPointer;
use super::{
    name::Name,
    version::{AnyVersion, Version},
};

use crate::core::project::Project;
use crate::core::project_archive::ProjectArchive;
use std::cmp::PartialOrd;
use std::hash::Hash;

#[derive(Debug)]
pub struct VersionItem<'a> {
    version: &'a Version,
    state: ProjectState,
}

impl<'a> VersionItem<'a> {
    pub fn new(v: &'a Version, s: ProjectState) -> Self {
        Self {
            version: v,
            state: s,
        }
    }

    pub fn get_version(&self) -> &Version {
        &self.version
    }

    pub fn get_state(&self) -> &ProjectState {
        &self.state
    }
}

impl<'a> PartialOrd for VersionItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.version.partial_cmp(other.version)
    }
}

impl<'a> Ord for VersionItem<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(other.version)
    }
}

impl<'a> PartialEq for VersionItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.version.eq(&other.version)
    }
}

impl<'a> Eq for VersionItem<'a> {}

impl<'a> Hash for VersionItem<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
        let _ = state.finish();
    }
}

#[derive(Debug)]
pub struct Catalog<'a> {
    inner: HashMap<Uuid, ProjectLevel>,
    mappings: HashMap<Name, Vec<Uuid>>,
    cache: Option<&'a PathBuf>,
    downloads: Option<&'a PathBuf>,
    available: Option<HashMap<&'a String, &'a PathBuf>>,
}

#[derive(Debug, PartialEq)]
pub enum ProjectState {
    Downloaded,
    Installation,
    Available,
    Unknown,
}

impl std::fmt::Display for ProjectState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Downloaded => write!(f, "download"),
            Self::Installation => write!(f, "install"),
            Self::Available => write!(f, "available"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug)]
pub struct ProjectLevel {
    installs: Vec<Project>,
    downloads: Vec<Project>,
    available: Vec<Project>,
}

impl ProjectLevel {
    pub fn new() -> Self {
        Self {
            installs: Vec::new(),
            available: Vec::new(),
            downloads: Vec::new(),
        }
    }

    pub fn add_install(&mut self, m: Project) -> bool {
        // only add if not a DST
        if m.is_dynamic() == false {
            self.installs.push(m);
            true
        } else {
            false
        }
    }

    pub fn add_download(&mut self, m: Project) -> bool {
        self.downloads.push(m);
        true
    }

    pub fn add_available(&mut self, m: Project) -> bool {
        self.available.push(m);
        true
    }

    pub fn get_installations(&self) -> &Vec<Project> {
        &self.installs
    }

    pub fn get_downloads(&self) -> &Vec<Project> {
        &self.downloads
    }

    pub fn get_availability(&self) -> &Vec<Project> {
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
    pub fn get_install(&self, version: &AnyVersion) -> Option<&Project> {
        Self::get_target_version(version, self.get_installations())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_download(&self, version: &AnyVersion) -> Option<&Project> {
        Self::get_target_version(version, self.get_downloads())
    }

    /// Returns the manifest with the most compatible version fitting `version`.
    pub fn get_available(&self, version: &AnyVersion) -> Option<&Project> {
        Self::get_target_version(version, self.get_availability())
    }

    /// References the project matching the most compatible version `version`.
    ///
    /// A `dev` version is only searched at the DEV_PATH. Any other version is
    /// first sought for in the cache installations, and if not found then searched
    /// for in the availability space.
    /// Note: `usable` to `false` will not check queued state
    pub fn get(
        &self,
        check_downloads: bool,
        check_available: bool,
        version: &AnyVersion,
    ) -> Option<&Project> {
        let ins = self.get_install(version);

        let dld = match check_downloads {
            true => self.get_download(version),
            false => None,
        };
        let ava = match check_available {
            true => self.get_available(version),
            false => None,
        };
        // keep the highest found version
        let highest = match ins {
            Some(i) => {
                let mut h = i;
                if let Some(d) = dld {
                    if d.get_man().get_project().get_version()
                        > i.get_man().get_project().get_version()
                    {
                        h = d;
                    }
                }
                if let Some(a) = ava {
                    if a.get_man().get_project().get_version()
                        > h.get_man().get_project().get_version()
                    {
                        h = a;
                    }
                }
                Some(h)
            }
            None => match dld {
                Some(d) => {
                    let mut h = d;
                    if let Some(a) = ava {
                        if a.get_man().get_project().get_version()
                            > h.get_man().get_project().get_version()
                        {
                            h = a;
                        }
                    }
                    Some(h)
                }
                None => ava,
            },
        };
        highest
    }

    /// Tracks what level the `manifest` came from.
    pub fn get_state(&self, project: &Project) -> ProjectState {
        if self.installs.iter().find(|f| f == &project).is_some() {
            ProjectState::Installation
        } else if self.available.iter().find(|f| f == &project).is_some() {
            ProjectState::Available
        } else if self.downloads.iter().find(|f| f == &project).is_some() {
            ProjectState::Downloaded
        } else {
            ProjectState::Unknown
        }
    }

    /// Finds the most compatible version matching `target` among the possible `space`.
    ///
    /// Returns `None` if no compatible version was found.
    ///
    /// Panics if a development version is entered as `target`.
    fn get_target_version<'a>(target: &AnyVersion, space: &'a Vec<Project>) -> Option<&'a Project> {
        // find the specified version for the given project
        let mut latest_version: Option<&Project> = None;
        space
            .iter()
            .filter(|prj| match &target {
                AnyVersion::Specific(v) => crate::core::version::is_compatible(
                    v,
                    prj.get_man().get_project().get_version(),
                ),
                AnyVersion::Latest => true,
            })
            .for_each(|prj| {
                if latest_version.is_none()
                    || prj.get_man().get_project().get_version()
                        > latest_version
                            .as_ref()
                            .unwrap()
                            .get_man()
                            .get_project()
                            .get_version()
                {
                    latest_version = Some(prj);
                }
            });
        latest_version
    }
}

impl<'a> Catalog<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            mappings: HashMap::new(),
            cache: None,
            downloads: None,
            available: None,
        }
    }

    /// Uses the cache slot name to check if the directory exists.
    pub fn is_cached_slot(&self, slot: &CacheSlot) -> bool {
        self.get_cache_path().join(slot.to_string()).is_dir()
    }

    /// Uses the download slot name to check if the file exists.
    pub fn is_downloaded_slot(&self, slot: &DownloadSlot) -> bool {
        self.get_downloads_path().join(slot.as_ref()).is_file()
    }

    pub fn get_downloaded_slot(&self, name: &Name, version: &Version) -> Option<DownloadSlot> {
        let mut ids = Vec::new();

        if let Ok(mut rd) = read_dir(self.get_downloads_path()) {
            let pat = format!("{}-", name);
            while let Some(d) = rd.next() {
                if let Ok(p) = d {
                    let file_name = p.file_name().into_string().unwrap();
                    // collect all possible UUIDs
                    if file_name.starts_with(&pat) == true {
                        ids.push(file_name.rsplit_once('-').unwrap().1.to_string());
                    }
                }
            }
        }
        match ids.len() {
            1 => Some(DownloadSlot(format!(
                "{}-{}-{}",
                name,
                version,
                ids.get(0).unwrap()
            ))),
            _ => None,
        }
    }

    /// Searches the `path` for projects installed.
    pub fn installations(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.cache = Some(&path);
        self.detect(path, &ProjectLevel::add_install, ProjectState::Installation)?;
        Ok(self)
    }

    /// Searches the `path` for projects downloaded.
    pub fn downloads(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.downloads = Some(&path);
        self.detect(path, &ProjectLevel::add_download, ProjectState::Downloaded)?;
        Ok(self)
    }

    /// Searches the `path` for projects available.
    pub fn available(mut self, channels: &HashMap<&'a String, &'a Channel>) -> Result<Self, Fault> {
        let mut map = HashMap::new();
        // update the availables
        for (&name, &chan) in channels {
            map.insert(name, chan.get_root());
            self.detect(
                map.get(name).unwrap(),
                &ProjectLevel::add_available,
                ProjectState::Available,
            )?;
        }
        self.available = Some(map);
        Ok(self)
    }

    pub fn set_cache_path(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.cache = Some(&path);
        Ok(self)
    }

    pub fn set_downloads_path(mut self, path: &'a PathBuf) -> Result<Self, Fault> {
        self.downloads = Some(&path);
        Ok(self)
    }

    pub fn inner(&self) -> &HashMap<Uuid, ProjectLevel> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<Uuid, ProjectLevel> {
        &mut self.inner
    }

    pub fn mappings(&self) -> &HashMap<Name, Vec<Uuid>> {
        &self.mappings
    }

    pub fn translate_name(&self, name: &PkgName) -> Result<Option<&ProjectLevel>, CodeFault> {
        if let Some(id) = name.get_uuid() {
            Ok(self.inner.get(id))
        } else {
            if let Some(cands) = self.mappings.get(&name.name) {
                match cands.len() {
                    0 => panic!("a mapping of name to uuid should already exist"),
                    1 => Ok(self.inner.get(cands.first().unwrap())),
                    _ => {
                        let mut conflicts = String::new();
                        cands.iter().enumerate().for_each(|(i, c)| {
                            conflicts.push_str(&format!(
                                "  candidate {}: {}+{}",
                                i + 1,
                                name.name,
                                c.encode()
                            ));
                            if i + 1 < cands.len() {
                                conflicts.push('\n');
                            }
                        });
                        Err(CodeFault(
                            None,
                            Box::new(Error::IpNamespaceCollision(
                                name.name.to_string(),
                                conflicts,
                                Hint::SolveNamespaceCollision,
                            )),
                        ))?
                    }
                }
            } else {
                // println!("{}", "here!");
                Err(CodeFault(
                    None,
                    Box::new(Error::IpNotFoundAnywhere(
                        name.name.to_string(),
                        Hint::CatalogList,
                    )),
                ))?
            }
        }
    }

    /// Returns all possible versions found for the `target` project.
    ///
    /// Returns `None` if the id is not found in the catalog.
    pub fn get_possible_versions(&self, id: &Uuid) -> Option<Vec<VersionItem<'_>>> {
        let kaban = self.inner.get(&id)?;
        let mut set = HashSet::new();
        // read from cache
        for project in kaban.get_installations() {
            set.insert(VersionItem::new(
                project.get_man().get_project().get_version(),
                ProjectState::Installation,
            ));
        }
        // read from downloads
        for project in kaban.get_downloads() {
            set.insert(VersionItem::new(
                project.get_man().get_project().get_version(),
                ProjectState::Downloaded,
            ));
        }
        // read from available
        for project in kaban.get_availability() {
            set.insert(VersionItem::new(
                project.get_man().get_project().get_version(),
                ProjectState::Available,
            ));
        }
        let mut arr: Vec<VersionItem> = set.into_iter().collect();
        arr.sort();
        arr.reverse();
        Some(arr)
    }

    pub fn update_installations(&mut self) -> () {
        todo!()
    }

    /// Searches the `path` for projects installed.
    pub fn refresh_installations(&mut self) -> Result<(), Fault> {
        if let Some(path) = self.cache {
            self.detect(path, &ProjectLevel::add_install, ProjectState::Installation)?;
        }
        Ok(())
    }

    /// Searches the `path` for projects downloaded.
    pub fn refresh_downloads(&mut self) -> Result<(), Fault> {
        if let Some(path) = self.downloads {
            self.detect(path, &ProjectLevel::add_download, ProjectState::Downloaded)?;
        }
        Ok(())
    }

    /// Finds all `Orbit.toml` manifest files (markings of a project) within the provided `path`.
    ///
    /// This function is generic enough to be used to catch projects at all 3 levels: dev, install, and available.
    fn detect(
        &mut self,
        path: &PathBuf,
        add: &dyn Fn(&mut ProjectLevel, Project) -> bool,
        lvl: ProjectState,
    ) -> Result<(), Fault> {
        match lvl {
            ProjectState::Installation => Project::detect_all(path, false),
            ProjectState::Available => ProjectPointer::detect_all(path),
            ProjectState::Downloaded => ProjectArchive::detect_all(path),
            ProjectState::Unknown => Ok(Vec::new()),
        }?
        .into_iter()
        // get the UUID for each manifest/project that was collected from the path finding
        .for_each(|project| match self.inner.get_mut(&project.get_uuid()) {
            // the UUID already exists in the catalog, so just add it in at its level
            Some(lvl) => {
                add(lvl, project);
                ()
            }
            // the UUID does not already exist in the catalog, so make a mapping
            None => {
                // verify the add was successful
                let mut lvl = ProjectLevel::new();
                let did_add = add(&mut lvl, project);
                // create a mapping for this uuid and insert into the catalog
                match did_add {
                    true => {
                        let project = lvl.get(true, true, &AnyVersion::Latest).unwrap();
                        let pkgpart = project.get_man().get_project().get_name();
                        // add this to the list of uuids for this name
                        match self.mappings.get_mut(pkgpart) {
                            Some(ids) => ids.push(project.get_uuid().clone()),
                            None => {
                                self.mappings
                                    .insert(pkgpart.clone(), vec![project.get_uuid().clone()]);
                            }
                        }
                        let pkgid = project.get_uuid().clone();
                        self.inner.insert(pkgid, lvl);
                    }
                    false => (),
                }
            }
        });
        Ok(())
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

type Checksum = String;

#[derive(PartialEq, Debug, Clone)]
pub struct CacheSlot(Uuid, Version, Checksum);

impl Display for CacheSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.0, self.1, self.2)
    }
}

impl CacheSlot {
    /// Combines the various components of a cache slot name into a `CacheSlot`.
    pub fn new(uuid: &Uuid, version: &Version, checksum: &Sha256Hash) -> Self {
        Self(uuid.clone(), version.clone(), checksum.to_string_short())
    }

    /// Creates a cache slot entry for a dynamic entry formed from a relative dependency
    pub fn new_rel(uuid: &Uuid, version: &Version) -> Self {
        Self(uuid.clone(), version.clone(), "relative".to_string())
    }

    /// Attempts to deconstruct a [String] into the components of a [CacheSlot].
    pub fn try_from_str(s: &str) -> Option<Self> {
        // split into three components
        let (uuid, rem) = s.split_once('-')?;
        let (version, sum) = rem.rsplit_once('-')?;
        Some(Self(
            match Uuid::from_str(uuid) {
                Ok(r) => r,
                Err(_) => return None,
            },
            match Version::from_str(version) {
                Ok(r) => r,
                Err(_) => return None,
            },
            sum.to_string(),
        ))
    }

    /// Checks if the given slot `other` belongs to the this cache slot (uuid
    /// and version equivalence check).
    pub fn is_child_slot(&self, other: &CacheSlot) -> bool {
        self.get_uuid() == other.get_uuid() && self.get_version() == other.get_version()
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn get_version(&self) -> &Version {
        &self.1
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct DownloadSlot(String);

impl DownloadSlot {
    /// Combines the various components of a project name into a [DownloadSlot].
    pub fn new(_name: &Name, uuid: &Uuid, version: &Version) -> Self {
        Self(format!("{}-{}.{}", uuid, version, ARCHIVE_EXT))
    }
}

impl AsRef<str> for DownloadSlot {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PointerSlot(String);

impl PointerSlot {
    /// Combines the various components of a cache slot name into a [PointerSlot].
    pub fn new(_name: &Name, uuid: &Uuid, version: &Version) -> Self {
        Self(format!("{}-{}", uuid, version))
    }
}

impl AsRef<str> for PointerSlot {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct PkgName<'a> {
    name: &'a Name,
    id: Option<&'a Uuid>,
}

impl<'a> PkgName<'a> {
    pub fn new(name: &'a Name, id: Option<&'a Uuid>) -> Self {
        Self { name: name, id: id }
    }

    pub fn get_name(&self) -> &'a Name {
        &self.name
    }

    pub fn get_uuid(&self) -> Option<&'a Uuid> {
        self.id
    }
}
