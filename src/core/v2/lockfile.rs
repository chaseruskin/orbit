use std::{str::FromStr, path::{PathBuf}};
use toml_edit::{Document, InlineTable, Formatted, Array};
use crate::{util::{sha256::Sha256Hash, anyerror::{AnyError, Fault}}, core::{pkgid::PkgPart, version::{Version, AnyVersion, self}, config::FromToml}};
use crate::core::{ip::IpSpec2, catalog::CacheSlot};

use super::manifest;
use super::manifest::Manifest;

pub const IP_LOCK_FILE: &str = "Orbit.lock";

use crate::core::v2::ip::Ip;

#[derive(Debug)]
pub struct LockFile(Vec<LockEntry>);

impl FromToml for LockFile {
    type Err = Fault;
    
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let mut inner = Vec::new();
        // take array as as tables
        if let Some(item) = table.get("ip") {
            match item.as_array_of_tables() {
                // parse each table entry into a `LockEntry` struct
                Some(arr) => {
                    for tbl in arr {
                        inner.push(LockEntry::from_toml(tbl)?);
                    }
                }
                None => {
                    return Err(AnyError(format!("expects 'ip' to be an array of tables")))?
                }
            }
        }
        Ok(Self(inner))
    }
}

impl LockFile {
    /// Creates a lockfile from a build list.
    pub fn from_build_list(build_list: &mut Vec<&Ip>) -> Self {
        // sort the build list by pkgid and then version
        build_list.sort_by(|&x, &y| { match x.get_man().get_ip().get_name().cmp(y.get_man().get_ip().get_name()) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => x.get_man().get_ip().get_version().cmp(y.get_man().get_ip().get_version()),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        } });
        
        Self(build_list.into_iter()
            .map(|ip| LockEntry::from(&ip.get_man(), &ip.get_root()))
            .collect())
    }

    /// Loads a lockfile from the `root` path.
    /// 
    /// If the file does not exist, then an empty lock entry list is returned.
    pub fn from_path(root: &PathBuf) -> Result<Self, Fault> {
        let lock_file = root.join(IP_LOCK_FILE);
        if lock_file.exists() == true {
            // open file
            let contents = std::fs::read_to_string(&lock_file)?;
            // parse toml syntax
            Ok(Self::from_toml(contents.parse::<Document>()?.as_table())?)
        } else {
            Ok(Self(Vec::new()))
        }
    }

    /// Returns an exact match of `target` and `version` from within the lockfile.
    pub fn get(&self, target: &PkgPart, version: &Version) -> Option<&LockEntry> {
        self.0.iter().find(|&f| &f.name == target && &f.version == version )
    }

    /// Returns the highest compatible version from the lockfile for the given `target`.
    pub fn get_highest(&self, target: &PkgPart, version: &AnyVersion) -> Option<&LockEntry> {
        // collect all versions
        let space: Vec<&Version> = self.0.iter().filter_map(|f| if &f.name == target { Some(&f.version) } else { None }).collect();
        match version::get_target_version(&version, &space) {
            Ok(v) => self.0.iter().find(|f| &f.name == target && f.version == v),
            Err(_) => None
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn inner(&self) -> &Vec<LockEntry> {
        &self.0
    } 
}

use crate::core::v2::manifest::Package;

#[derive(Debug, PartialEq)]
pub struct LockEntry {
    name: manifest::Id,
    version: manifest::Version,
    sum: Option<Sha256Hash>,
    source: Option<manifest::Source>,
    dependencies: Option<Vec<IpSpec2>>,
}

impl LockEntry {
    fn from(man: &Manifest, dir: &PathBuf) -> Self {
        Self {
            name: man.get_ip().get_name().clone(), 
            version: man.get_ip().get_version().clone(), 
            sum: Some(Package::read_checksum_proof(&dir).unwrap_or(Package::compute_checksum(&dir))), 
            source: man.get_ip().get_source().clone(),
            dependencies: if let Some(deps) = man.get_deps() {
                match deps.len() {
                    0 => None,
                    _ => Some({
                        let mut result: Vec<IpSpec2> = deps
                            .into_iter()
                            .map(|e| { IpSpec2::from((e.0.clone(), e.1.clone())) })
                            .collect();
                        result.sort_by(|x, y| { match x.get_name().cmp(&y.get_name()) {
                            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                            std::cmp::Ordering::Equal => x.get_version().cmp(&y.get_version()),
                            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                        } });
                        result
                    }),
                }
            } else {
                None
            }
        }
    }
}

impl LockEntry {
    /// Performs an equality check against a target entry `other`.
    /// 
    /// Ignores the checksum comparison because the target ip should not have its
    /// checksum computed in the .lock file.
    pub fn matches_target(&self, other: &LockEntry) -> bool {
        self.get_name() == other.get_name() && 
        self.get_version() == other.get_version() &&
        self.get_source() == other.get_source() &&
        self.get_deps() == other.get_deps()
    }

    pub fn get_deps(&self) -> Option<&Vec<IpSpec2>> {
        self.dependencies.as_ref()
    }

    pub fn get_sum(&self) -> Option<&Sha256Hash> {
        self.sum.as_ref()
    }

    pub fn get_source(&self) -> Option<&manifest::Source> {
        self.source.as_ref()
    }

    pub fn get_name(&self) -> &manifest::Id {
        &self.name
    }

    pub fn get_version(&self) -> &manifest::Version {
        &self.version
    }

    pub fn to_cache_slot_key(&self) -> CacheSlot {
        CacheSlot::new(self.get_name(), self.get_version(), self.get_sum().unwrap())
    }

    pub fn to_toml(&self, table: &mut toml_edit::Table) -> () {
        table["name"] = toml_edit::value(&self.name.to_string());
        table["version"] = toml_edit::value(&self.version.to_string());
        if let Some(sum) = self.get_sum() {
            table["sum"] = toml_edit::value(&sum.to_string());
        }
        if let Some(src) = self.get_source() {
            table["source"] = toml_edit::value(src.to_string());
        }
        if let Some(deps) = &self.dependencies {
            table.insert("dependencies", toml_edit::Item::Value(toml_edit::Value::Array(Array::new())));
            for entry in deps {
                let mut inline = InlineTable::new();
                // @TODO write newlines after each item?
                inline.insert("name", toml_edit::Value::String(Formatted::new(entry.get_name().to_string())));
                inline.insert("version", toml_edit::Value::String(Formatted::new(entry.get_version().to_string())));
                inline.decor_mut().set_prefix("\n    ");
                table["dependencies"].as_array_mut()
                    .unwrap()
                    .push_formatted(toml_edit::Value::InlineTable(inline));
            }
            table["dependencies"].as_array_mut().unwrap().set_trailing("\n");
        }
    }

    pub fn to_ip_spec(&self) -> IpSpec2 {
        IpSpec2::new(self.name.clone(), self.version.clone())
    }
}

impl FromToml for LockEntry {
    type Err = Fault; 

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            name: PkgPart::from_str(table.get("name").unwrap().as_str().unwrap())?,
            version: Version::from_str(table.get("version").unwrap().as_str().unwrap())?,
            sum: match table.get("sum") {
                Some(item) => Some(Sha256Hash::from_str(item.as_str().unwrap())?),
                None => None,
            },
            source: if let Some(src) = table.get("source") { Some(manifest::Source::from_str(src.as_str().unwrap())?) } else { None },
            dependencies: {
                match table.get("dependencies") {
                    Some(item) => {
                        let mut result: Vec<IpSpec2> = Vec::new();
                        for entry in item.as_array().unwrap() {
                            let entry = entry.as_inline_table().unwrap();
                            result.push(
                                IpSpec2::new(
                                    PkgPart::from_str(entry["name"].as_str().unwrap()).unwrap(),
                                    Version::from_str(entry["version"].as_str().unwrap()).unwrap(),
                                )
                            );
                        }
                        Some(result)
                    },
                    None => None,
                }
            }
        })
    }
}