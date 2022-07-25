use toml_edit::{Document, Table, ArrayOfTables, Array, value};
use std::collections::HashMap;
use std::io::Write;
use std::path;
use std::path::PathBuf;
use std::error::Error;
use crate::core::pkgid::PkgId;
use crate::core::lockfile::{LockFile, IP_LOCK_FILE};
use crate::util::anyerror::{AnyError, Fault};
use crate::util::sha256::{Sha256Hash, self};
use crate::util::url::Url;
use std::str::FromStr;
use crate::core::version::Version;
use crate::util::filesystem::normalize_path;

use super::config::{FromToml, FromTomlError};
use super::lockfile::LockEntry;
use super::ip::IpSpec;
use super::version::AnyVersion;
use super::vhdl::primaryunit::PrimaryUnit;
use super::vhdl::token::{Identifier, IdentifierError};

/// Takes an iterative approach to iterating through directories to find a file
/// matching `name`.
/// 
/// Note: `name` is become a glob-style pattern.
/// 
/// Stops descending the directories upon finding first match of `name`. 
/// The match must be case-sensitive. If `is_exclusive` is `false`, then the directory
/// with match will continued to be searched at that level and then re-track.
fn find_file(path: &PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<PathBuf>, Fault> {
    // create a glob-style pattern
    let pattern = glob::Pattern::new(name).unwrap();
    // list of directories to continue to process
    let mut to_process: Vec<PathBuf> = Vec::new();
    let mut result = Vec::new();
    // start at base path
    if path.is_dir() {
        to_process.push(path.to_path_buf());
    // only look at file and exit
    } else if path.is_file() && pattern.matches(path.file_name().unwrap().to_str().unwrap()) {
        return Ok(vec![path.to_path_buf()])
    }
    // process next directory to read
    while let Some(entry) = to_process.pop() {
        // needs to look for more clues deeper in the filesystem
        if entry.is_dir() {
            let mut next_to_process = Vec::new();
            let mut found_file = false;
            // iterate through all next-level directories for potential future processing
            for e in std::fs::read_dir(entry)? {
                let e = e?;
                if pattern.matches(e.file_name().to_str().unwrap()) {
                    result.push(e.path());
                    found_file = true;
                    if is_exclusive == true {
                        break;
                    }
                } else if e.file_type().unwrap().is_dir() == true {
                    next_to_process.push(e.path());
                }
            }
            // add next-level directories to process
            if found_file == false {
                to_process.append(&mut next_to_process);
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]
pub struct Manifest {
    // track where the file loads/stores from
    path: path::PathBuf, 
    // maintain the data
    document: Document
}

impl Manifest {
    /// Creates a new empty `Manifest` struct.
    pub fn new() -> Self {
        Self {
            path: path::PathBuf::new(),
            document: Document::new(),
        }
    }

    /// Reads from the file at `path` and parses into a valid toml document for a `Manifest` struct. 
    /// 
    /// Errors if the file does not exist or the TOML parsing fails.
    pub fn from_path(path: PathBuf) -> Result<Self, Fault> {
        if std::path::Path::exists(&path) == false {
            return Err(AnyError(format!("missing manifest file {:?}", path)))?
        }
        Ok(Self {
            // load the data as a string
            document: std::fs::read_to_string(&path)?.parse::<Document>()?,
            path: path,     
        })
    }

    /// Finds all Manifest files available in the provided path `path`.
    /// 
    /// Errors if on filesystem problems.
    pub fn detect_all(path: &std::path::PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<Manifest>, Fault> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file
        for entry in find_file(&path, &name, is_exclusive)? {
            // read ip_spec from each manifest
            result.push(Manifest::from_path(entry)?);
        }
        Ok(result)
    }

    /// Reads a value from the manifest file.
    /// 
    /// If the key does not exist, it will return `None`. Assumes the key already is a string if it does
    /// exist.
    pub fn read_as_str(&self, table: &str, key: &str) -> Option<String> {
        if let Some(item) = self.document[table].get(key) {
            Some(item.as_str().unwrap().to_string())
        } else {
            None
        }
    }

    /// Edits the .toml document at the `table`.`key` with `value`.
    /// 
    pub fn write<T: ToString>(&mut self, table: &str, key: &str, value: T) -> ()
    where toml_edit::Value: From<T> {
        self.document[table][key] = toml_edit::value(value);
    }

    /// Stores data to file from `Manifest` struct.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.path, self.document.to_string())?;
        Ok(())
    }

    pub fn get_path(&self) -> &path::PathBuf {
        &self.path
    }

    pub fn get_mut_doc(&mut self) -> &mut Document {
        &mut self.document
    }

    pub fn get_doc(&self) -> &Document {
        &self.document
    }
}

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";
pub const IP_MANIFEST_PATTERN_FILE : &str = "Orbit-*.toml";
const DEPENDENCIES_KEY: &str = "dependencies";
pub const ORBIT_SUM_FILE: &str = ".orbit-checksum";
pub const ORBIT_METADATA_FILE: &str = ".orbit-metadata";

#[derive(Debug)]
pub struct IpManifest{ 
    manifest: Manifest,
    ip: IpToml,
}

impl PartialEq for IpManifest {
    fn eq(&self, other: &Self) -> bool {
        self.ip == other.ip
    }

    fn ne(&self, other: &Self) -> bool {
        self.ip != other.ip
    }
}

#[derive(Debug, PartialEq)]
pub struct IpToml {
    ip: Ip,
    deps: DependencyTable,
}

impl IpToml {
    pub fn new() -> Self {
        Self { ip: Ip::new(), deps: DependencyTable::new() }
    }
}

impl std::fmt::Display for IpManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = match self.get_repository() {
            Some(r) => r.to_string(),
            None => String::new(),
        };  
        write!(f, "\
ip:         {}
summary:    {}
version:    {}
repository: {}
size:       {:.2} MB
dependencies:
{}", 
self.get_pkgid(), 
self.get_summary().unwrap_or(&"".to_string()), 
self.get_version(),
url,
crate::util::filesystem::compute_size(&self.manifest.get_path().parent().unwrap(), crate::util::filesystem::Unit::MegaBytes).unwrap(),
self.get_dependencies().to_string()
    )}
}

#[derive(Debug, PartialEq)]
pub struct Ip {
    name: PkgId,
    version: Version,
    repository: Option<Url>,
    summary: Option<String>,
    changelog: Option<String>,
    readme: Option<String>,
    units: Option<Vec<Identifier>>,
}

impl Ip {
    pub fn new() -> Self {
        Self { 
            name: PkgId::new(), 
            version: Version::new(), 
            repository: None, 
            summary: None, 
            changelog: None, 
            readme: None,
            units: None,
        }
    }

    pub fn get_repository(&self) -> Option<&Url> {
        self.repository.as_ref()
    }

    pub fn get_version(&self) -> &Version {
        &self.version
    }

    pub fn get_pkgid(&self) -> &PkgId {
        &self.name
    }

    pub fn get_summary(&self) -> Option<&String> {
        self.summary.as_ref()
    }
}

impl FromToml for Ip {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            name: {
                let name: String = Self::require(table, "name")?;
                let library: String = Self::require(table, "library")?;
                let vendor: String = Self::require(table, "vendor")?;
                PkgId::new().name(&name)?.library(&library)?.vendor(&vendor)?
            },
            version: Self::require(table, "version")?,
            repository: Self::get(table, "repository")?,
            summary: Self::get(table, "summary")?,
            changelog: Self::get(table, "changelog")?,
            readme: Self::get(table, "readme")?,
            units: match table.get("units") {
                Some(i) => match i.as_array() {
                    Some(arr) => {
                        let result: Result<Vec<_>, IdentifierError> = arr.into_iter()
                            .filter_map(|f| f.as_str() )
                            .map(|f| f.to_owned().parse::<Identifier>() )
                            .collect();
                        Some(result?)
                    }
                    None => return Err(FromTomlError::ExpectingStringArray("units".to_owned()))?,
                }
                None => None,
            },
        })
    }
}

impl FromToml for IpToml {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        // grab the ip table
        let ip = if let Some(item) = table.get("ip") {
            match item.as_table() {
                Some(tbl) => Ip::from_toml(tbl)?,
                None => return Err(AnyError(format!("expects key 'ip' to be a toml table")))?
            }
        } else {
            return Err(FromTomlError::MissingEntry("ip".to_string()))?
        };
        // grab the dependencies table
        let dt = if let Some(item) = table.get("dependencies") {
            match item.as_table() {
                Some(tbl) => DependencyTable::from_toml(tbl)?,
                None => return Err(AnyError(format!("expects key 'dependencies' to be a toml table")))?
            }
        } else {
            DependencyTable::new()
        };
        Ok(Self {
            ip: ip,
            deps: dt,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct DependencyTable(HashMap<PkgId, AnyVersion>);

impl std::fmt::Display for DependencyTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for pair in &self.0 {
            write!(f, "    {} {}\n", pair.0, pair.1)?
        }
        Ok(())
    }
}

impl DependencyTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn inner(&self) -> &HashMap<PkgId, AnyVersion> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<PkgId, AnyVersion> {
        &mut self.0
    }

    pub fn insert(&mut self, pkg: PkgId, ver: AnyVersion) -> Option<AnyVersion> {
        // overwrite the existing key
        self.0.insert(pkg, ver)
    }

    pub fn as_sorted_vec(&self) -> Vec<(&PkgId, &AnyVersion)> {
        let mut deps: Vec<(&PkgId, &AnyVersion)> = self.0.iter().map(|f| { f }).collect();
        // sort the dependencies
        deps.sort_by(|x, y| { match x.0.cmp(&y.0) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => x.1.cmp(&y.1),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        } });
        deps
    }
}

impl FromToml for DependencyTable {
    type Err = Fault;
    
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let mut map = HashMap::new();
        // traverse three tables deep to retrieve V.L.N
        for (vendor, v_item) in table.iter() {
            let mut switch = true;
            for (library, l_item) in v_item.as_table().unwrap_or(&toml_edit::Table::default()) {
                switch = true;
                for (name, n_item) in l_item.as_table().unwrap_or(&toml_edit::Table::default()) {
                    switch = false;
                    let pkgid = PkgId::new().name(name)?.library(library)?.vendor(vendor)?; 
                    // create version
                    let version = match n_item.as_str() {
                        Some(s) => AnyVersion::from_str(s)?,
                        None => return Err(FromTomlError::ExpectingString(format!("{}.{}.{}", vendor, library, name)))?
                    };
                    // insert into lut
                    if let Some(prev) = map.insert(pkgid.clone(), version.clone()) {
                        return Err(AnyError(format!("ip '{}' cannot be a direct dependency more than once\n\nUse only one of the listed versions: '{}' or '{}'", pkgid, version, prev)))?
                    }
                }
                if switch == true { return Err(AnyError(format!("partial ip pkgid key '{}.{}.' in dependencies table", vendor, library)))? }
            }
            if switch == true { return Err(AnyError(format!("partial ip pkgid key '{}.' in dependencies table", vendor)))? }
        }
        Ok(Self(map))
    }
}

impl IpManifest {
    /// Creates an empty `IpManifest` struct.
    pub fn new() -> Self {
        IpManifest {
            manifest: Manifest::new(),
            ip: IpToml::new(),
        }
    }

    /// Initializes a lock file from the manifest root path. 
    pub fn into_lockfile(&self) -> Result<LockFile, Fault> {
        LockFile::from_path(&self.get_root())
    }

    pub fn read_units_from_metadata(&self) -> Option<Vec<PrimaryUnit>> {
        let meta_file = self.get_root().join(ORBIT_METADATA_FILE);
        if std::path::Path::exists(&meta_file) == true {
            if let Ok(contents) = std::fs::read_to_string(&meta_file) {
                if let Ok(toml) = contents.parse::<Document>() {
                    let entry = toml.get("ip")?.as_table()?.get("units")?.as_array()?;
                    Some(entry.into_iter()
                        .filter_map(|p| PrimaryUnit::from_toml(p.as_inline_table().unwrap())).collect())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Clones into a new `IpSpec` struct.
    pub fn into_ip_spec(&self) -> IpSpec {
        IpSpec::new(self.get_pkgid().clone(), self.get_version().clone())
    }

    /// Gathers the list of primary design units for the current ip.
    /// 
    /// If the manifest has an toml entry for `units`, it will return that list rather than go through files.
    pub fn collect_units(&self) -> Vec<PrimaryUnit> {
        // try to read from metadata file
        match self.read_units_from_metadata() {
            // use precomputed result
            Some(units) => units,
            None => {
                // collect all files
                let files = crate::util::filesystem::gather_current_files(&self.get_manifest().get_path().parent().unwrap().to_path_buf());
                crate::core::vhdl::primaryunit::collect_units(&files).into_iter().map(|e| e.0).collect()
            }
        }
    }

    /// Attempts to first read from units key entry if exists in toml file.
    pub fn read_units(&self) -> Option<Vec<PrimaryUnit>> {
        let entry = self.get_manifest().get_doc().get("ip")?.as_table()?.get("units")?.as_array()?;
        Some(entry.into_iter()
            .filter_map(|p| PrimaryUnit::from_toml(p.as_inline_table().unwrap())).collect())
    }

    pub fn get_root(&self) -> std::path::PathBuf {
        self.get_manifest().get_path().parent().unwrap().to_path_buf()
    }

    /// Finds all IP manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Manifest::detect_all(path, IP_MANIFEST_FILE, true)?.into_iter().map(|f| IpManifest::from_manifest(f)).collect()
    }

    /// Finds all IP manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_available(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Manifest::detect_all(path, IP_MANIFEST_PATTERN_FILE, false)?.into_iter().map(|f| IpManifest::from_manifest(f)).collect()
    }

    /// Creates a new minimal IP manifest for `path`.
    /// 
    /// Does not actually write the data to `path`. Use the `fn save` to write to disk.
    pub fn init(path: path::PathBuf) -> Self {
        let toml = BARE_MANIFEST.parse::<Document>().unwrap();
        Self { 
            ip: FromToml::from_toml(&toml.as_table()).unwrap(),
            manifest: Manifest {
                path: path,
                document: toml,
            },
        }
    }

    /// Deletes the project and its files found at the root path.
    pub fn remove(&self) -> Result<(), Fault> {
        std::fs::remove_dir_all(self.get_root())?;
        Ok(())
    }

    /// Computes the checksum on the root of the IP.
    /// 
    /// Changes the current working directory to the root for consistent computation.
    pub fn compute_checksum(&self) -> Sha256Hash {
        let cd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&self.get_root()).unwrap();
        let ip_files = crate::util::filesystem::gather_current_files(&PathBuf::from("."));
        let checksum = crate::util::checksum::checksum(&ip_files);
        std::env::set_current_dir(&cd).unwrap();
        checksum
    }

    /// Writes a metadata file for internal usage within orbit.
    pub fn write_metadata(&mut self) -> Result<(), Fault> {
        // write units to document
        self.stash_units();
        // write the document to .orbit-metadata
        let mut meta = std::fs::File::create(self.get_root().join(ORBIT_METADATA_FILE))?;
        meta.write(self.get_manifest().get_doc().to_string().as_bytes())?;
        Ok(())
    }

    /// Gets the already calculated checksum from an installed IP from '.orbit-checksum'.
    /// 
    /// This fn can return the different levels of the check-sum, whether its the dynamic
    /// SHA (level 1) or the original SHA (level 0).
    /// 
    /// Returns `None` if the file does not exist, is unable to read into a string, or
    /// if the sha cannot be parsed.
    pub fn get_checksum_proof(&self, level: u8) -> Option<Sha256Hash> {
        let sum_file = self.get_root().join(ORBIT_SUM_FILE);
        if sum_file.exists() == false {
            None
        } else {
            match std::fs::read_to_string(&sum_file) {
                Ok(text) => {
                    let mut sums = text.split_terminator('\n').skip(level.into());
                    match sha256::Sha256Hash::from_str(&sums.next().expect("level was out of bounds")) {
                        Ok(sha) => Some(sha),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }

    /// Attempts to load the ip's computed checksum from lockfile (Orbit.lock.)
    pub fn load_checksum_from_lock(&self) -> Option<Sha256Hash> {
        let lock_path = self.get_root().join(IP_LOCK_FILE);
        // check that the file exists
        if lock_path.exists() == false || lock_path.is_file() == false {
            return None
        }

        // look up the checksum in the .lock file to compare
        let lock = match LockFile::from_path(&self.get_root()) {
            Ok(l) => l,
            Err(_) => return None,
        };
        // verify the ip is in the .lock file (will bypass if entry is not there)
        match lock.get(self.get_pkgid(), self.get_version()) {
            Some(it) => Some(it.get_sum()?.clone()),
            None => None,
        }
    }

    /// Saves the build list to disk as IP_LOCK_FILE.
    /// 
    /// The lock file helps fill in the missing pieces of the puzzle when a different
    /// environment/machine is attempting to rebuild a project.
    /// 
    /// Note: Internally, this function will sort the build into alphabetical order.
    pub fn write_lock(&self, build_list: &mut Vec<&IpManifest>) -> Result<(), Fault> {
        let lock_file = self.get_root().join(IP_LOCK_FILE);
        // remove any old stale lock file
        if lock_file.exists() == true {
            std::fs::remove_file(&lock_file)?;
        }
        // write the new lock file
        let mut lock = std::fs::File::create(lock_file)?;
        lock.write(LOCK_HEADER.as_bytes())?;
        // load as toml and save as toml
        let mut toml = Document::new();
        toml["ip"] = toml_edit::Item::ArrayOfTables(ArrayOfTables::new());
        let lock_table = toml["ip"].as_array_of_tables_mut().unwrap();

        // sort the build list by pkgid and then version
        build_list.sort_by(|&x, &y| { match x.get_pkgid().cmp(y.get_pkgid()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => x.get_version().cmp(y.get_version()),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        } });
        
        build_list.into_iter().for_each(|ip| {
            let mut table = Table::new();
            LockEntry::from(*ip).to_toml(&mut table);
            if *ip == self {
                table.remove_entry("sum");
            }
            lock_table.push(table);
        });
        // write to disk
        lock.write(toml.to_string().as_bytes())?;
        Ok(())
    }

    /// Updates the dependencies table.
    pub fn insert_dependency(&mut self, pkgid: PkgId, ver: AnyVersion) -> Option<AnyVersion> {
        if self.get_manifest().get_doc().as_table().contains_table(DEPENDENCIES_KEY) == false {
            self.get_manifest_mut().get_mut_doc()[DEPENDENCIES_KEY] = toml_edit::Item::Table(Table::new());
        }
        let prev_value = self.ip.deps.inner_mut().remove(&pkgid);
        self.get_manifest_mut().get_mut_doc()
            [DEPENDENCIES_KEY]
            [&pkgid.get_vendor().as_ref().unwrap().to_string()]
            [&pkgid.get_library().as_ref().unwrap().to_string()]
            [&pkgid.get_name().to_string()] = toml_edit::value(&ver.to_string());

        self.get_manifest_mut().get_mut_doc()
            [DEPENDENCIES_KEY]
            [&pkgid.get_vendor().as_ref().unwrap().to_string()].as_inline_table_mut().map(|f| f.set_dotted(true));

        self.get_manifest_mut().get_mut_doc()
            [DEPENDENCIES_KEY]
            [&pkgid.get_vendor().as_ref().unwrap().to_string()]
            [&pkgid.get_library().as_ref().unwrap().to_string()].as_inline_table_mut().map(|f| f.set_dotted(true));
        prev_value
    }

    /// Creates a new IP at the `path`.
    /// 
    /// A manifest is created one level within `path` as IP_MANIFEST_FILE.
    /// Assumes the `pkgid` is fully qualified. Saves the manifest to disk.
    pub fn create(path: std::path::PathBuf, pkgid: &PkgId, force: bool, init: bool) -> Result<Self, Box<dyn Error>> {
        if std::path::Path::exists(&path) == true {
            // remove the entire existing directory
            if force == true {
                std::fs::remove_dir_all(&path)?;
            // error if directories exist
            } else if init == false {
                return Err(Box::new(AnyError(format!("failed to create new ip because directory '{}' already exists", path.display()))))
            }
        }
        // create all directories if the do not exist
        std::fs::create_dir_all(&path)?;

        // @TODO issue warning if the path it was placed is outside of DEV_PATH or if DEV_PATH is not set

        let toml = BARE_MANIFEST.parse::<Document>().unwrap();
        let mut ip_man = Self { 
            ip: FromToml::from_toml(&toml.as_table()).unwrap(),
            manifest: Manifest {
                path: path.join(IP_MANIFEST_FILE),
                document: toml,
            },
        };
        // fill in fields
        ip_man.get_manifest_mut().write("ip", "name", pkgid.get_name());
        ip_man.get_manifest_mut().write("ip", "library", pkgid.get_library().as_ref().unwrap());
        ip_man.get_manifest_mut().write("ip", "vendor", pkgid.get_vendor().as_ref().unwrap());
        // save the manifest
        ip_man.get_manifest_mut().save()?;

        // create an empty git repository
        git2::Repository::init(&path)?;

        Ok(ip_man)
    }
    
    /// Determines if a new .lock file needs to be generated for the current ip.
    /// 
    /// Returning `true` signifies the .lock file is up-to-date. Assumes the function
    /// is called from the ip's root directory.
    /// 
    /// Conditions for re-solving:
    /// - Orbit.lock does not exist at root IP directory level
    /// - Orbit.lock is missing current IP's checksum
    /// - Orbit.lock has an outdated checksum
    pub fn is_locked(&self) -> bool {
        match self.load_checksum_from_lock() {
            // verify the checksums match
            Some(key) => {
                println!("{} = {}", key, self.compute_checksum());
                key == self.compute_checksum() 
            },
            None => false
        }
    }

    /// Checks if needing to read off the lock file.
    /// 
    /// This determines if the lock file's data matches the Orbit.toml manifest data,
    /// indicating it is safe to pull data from the lock file and no changes would be
    /// made to the lock file.
    pub fn can_use_lock(&self) -> bool {
        match self.into_lockfile() {
            Ok(lock) => match lock.get(self.get_pkgid(), self.get_version()) {
                // okay to use the lock file if the entry is the same as manifest
                Some(entry) => entry.matches_target(&LockEntry::from(self)),
                None => return false,
            }
            Err(_) => return false,
        }
    }

    pub fn get_pkgid(&self) -> &PkgId {
        &self.ip.ip.get_pkgid()
    }

    pub fn into_pkgid(self) -> PkgId {
        self.ip.ip.name
    }

    pub fn get_version(&self) -> &Version {
        &self.ip.ip.get_version()
    }

    pub fn get_summary(&self) -> Option<&String> {
        self.ip.ip.get_summary()
    }

    pub fn get_manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }

    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Loads data from file as a `Manifest` struct. 
    /// 
    /// Errors on parsing errors for toml and errors on any particular rules for
    /// manifest formatting/required keys.
    fn from_manifest(m: Manifest) -> Result<Self, Box<dyn Error>> {
        Ok(IpManifest { ip: Self::wrap_toml(&m, IpToml::from_toml(&m.get_doc().as_table()))?, manifest: m, })
    }

    /// Loads an existing `IpManifest` from `path` by reading it as a TOML file. 
    /// 
    /// Assumes `path` is the root of the ip project. The `IP_MANIFEST_FILE` is assumed
    /// to be on located directly within the `path`.
    pub fn from_path(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let man = Manifest::from_path(path.join(IP_MANIFEST_FILE))?;
        Ok(Self {
            ip: Self::wrap_toml(&man, IpToml::from_toml(man.get_doc().as_table()))?,
            manifest: man,
        })
    }

    /// Creates a string for printing an ip manifest to during `orbit tree`. 
    pub fn to_ip_spec(&self) -> IpSpec {
        IpSpec::new(self.get_pkgid().clone(), self.get_version().clone())
    }

    pub fn get_dependencies(&self) -> &DependencyTable {
        &self.ip.deps
    }

    pub fn get_repository(&self) -> Option<&Url> {
        self.ip.ip.get_repository()
    }

    fn wrap_toml<T, E: std::fmt::Display>(m: &Manifest, r: Result<T, E>) -> Result<T, impl std::error::Error> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(AnyError(format!("manifest {}: {}", normalize_path(m.get_path().clone()).display(), e))),
        }
    }

    /// Caches the result of collecting all the primary desgin units for the given package.
    /// 
    /// Writes the data to the toml data structure. Note, this function does not save the manifest data to file.
    pub fn stash_units(&mut self) -> () {
        // collect the units
        let units = self.collect_units();
        let tbl = self.get_manifest_mut().get_mut_doc()["ip"].as_table_mut().unwrap();
        tbl.insert("units", toml_edit::Item::Value(toml_edit::Value::Array(Array::new())));
        let arr = tbl["units"].as_array_mut().unwrap();
        // map the units into a serialized data format
        for unit in &units {
            arr.push(unit.to_toml());
        }
        tbl["units"].as_array_mut().unwrap().iter_mut().for_each(|f| {
            f.decor_mut().set_prefix("\n    ");
            f.decor_mut().set_suffix("");
        });
        tbl["units"].as_array_mut().unwrap().set_trailing("\n");
    }


    pub fn generate_dst_lut(&self) -> HashMap<Identifier, String> {
        let units = self.read_units_from_metadata().unwrap();
        let checksum = self.get_checksum_proof(0).unwrap();
        // compose the lut for symbol transformation
        let mut lut = HashMap::new();
        units.into_iter().for_each(|f| {
            lut.insert(
                f.as_iden().unwrap().clone(), 
                "_".to_string() + checksum.to_string().get(0..10).unwrap()
            );
        });
        lut
    }

    /// Checks the metadata file for a entry for `dynamic`.
    pub fn is_dynamic(&self) -> bool {
        let meta_path = self.get_root().join(ORBIT_METADATA_FILE);
        let table = if std::path::Path::exists(&meta_path) == true {
            let contents = std::fs::read_to_string(meta_path).unwrap();
            contents.parse::<Document>().unwrap()
        } else {
            return false
        };
        match table.get("dynamic") {
            Some(item) => match item.as_bool() {
                Some(b) => b,
                None => false,
            },
            None => false,
        }
    }

    /// Adds to manifest file to set as dynamic.
    pub fn set_as_dynamic(&mut self) -> () {
        self.get_manifest_mut().get_mut_doc().as_table_mut()["dynamic"] = value(true);
    }

}

const BARE_MANIFEST: &str = "\
[ip]
name    = \"\"
library = \"\"
version = \"0.1.0\"
vendor  = \"\"

# To learn more about writing the manifest, see https://c-rus.github.io/orbit/4_topic/2_orbittoml.html

[dependencies]
";

const LOCK_HEADER: &str = "\
# This file is automatically generated by Orbit.
# Do not manually edit this file.
";

#[cfg(test)]
mod test {
    use crate::core::version::PartialVersion;
    use super::*;
    use std::str::FromStr;

    #[test]
    fn new() {
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = IpManifest::init(m.path().to_path_buf());
        assert_eq!(manifest.manifest.document.to_string(), BARE_MANIFEST);
    }

    #[test]
    fn from_toml() {
        let toml_code = r#"
[ip]
"#;
        // missing all required fields
        let manifest = Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: toml_code.parse::<Document>().unwrap()
        };
        assert_eq!(IpManifest::from_manifest(manifest).is_err(), true);

        // missing 'version' key
        let toml_code = r#"
[ip]
vendor = "v"
library = "l"
name = "n"
"#;
        // missing all required fields
        let manifest = Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: toml_code.parse::<Document>().unwrap()
        };
        assert_eq!(IpManifest::from_manifest(manifest).is_err(), true);
    }

    #[test]
    fn deps() {
        // empty `dependencies` table
        let toml_code = r#"
[ip]
name = "gates"

[dependencies]
"#;
        // empty table
        let doc = toml_code.parse::<Document>().unwrap();
        assert_eq!(DependencyTable::from_toml(doc.as_table().get("dependencies").unwrap().as_table().unwrap()).unwrap(), DependencyTable::new());

        // `dependencies` table with entries
        let toml_code = r#"
[dependencies] 
ks-tech.rary.gates = "1.0.0"
ks-tech.util.toolbox = "2"
c-rus.eel4712c.lab1 = "4.2"
"#;
        // empty table
        let doc = toml_code.parse::<Document>().unwrap();
        let mut map = HashMap::new();
        map.insert(PkgId::from_str("ks-tech.rary.gates").unwrap(), AnyVersion::Specific(PartialVersion::new().major(1).minor(0).patch(0)));
        map.insert(PkgId::from_str("ks-tech.util.toolbox").unwrap(),  AnyVersion::Specific(PartialVersion::new().major(2)));
        map.insert(PkgId::from_str("c-rus.eel4712c.lab1").unwrap(),  AnyVersion::Specific(PartialVersion::new().major(4).minor(2)));

        assert_eq!(DependencyTable::from_toml(doc.as_table().get("dependencies").unwrap().as_table().unwrap()).unwrap(), DependencyTable(map));
    }
}