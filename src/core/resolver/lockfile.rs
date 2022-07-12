use std::{str::FromStr, path::{PathBuf}};
use toml_edit::{Document, InlineTable, Formatted, Array};
use crate::{util::{sha256::Sha256Hash, anyerror::{AnyError, Fault}}, core::{pkgid::PkgId, version::{Version, AnyVersion, self}, config::FromToml, manifest::IpManifest}};

type Module = (PkgId, AnyVersion);

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

    pub fn get(&self, target: &PkgId, version: &Version) -> Option<&LockEntry> {
        self.0.iter().find(|&f| &f.name == target && &f.version == version )
    }

    pub fn get_highest(&self, target: &PkgId, version: &AnyVersion) -> Option<&LockEntry> {
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

#[derive(Debug, PartialEq)]
struct Source(String);

#[derive(Debug, PartialEq)]
pub struct LockEntry {
    name: PkgId,
    version: Version,
    sum: Option<Sha256Hash>,
    source: Source,
    dependencies: Option<Vec<Module>>,
}

impl From<&IpManifest> for LockEntry {
    fn from(ip: &IpManifest) -> Self {
        Self {
            name: ip.get_pkgid().clone(), 
            version: ip.get_version().clone(), 
            sum: Some(ip.get_checksum_proof(0).unwrap_or(ip.compute_checksum())), 
            source: Source(ip.get_repository().unwrap_or(&String::new()).to_string()),
            dependencies: match ip.get_dependencies().inner().len() {
                0 => None,
                _ => Some({
                    let mut result: Vec<(PkgId, AnyVersion)> = ip.get_dependencies()
                        .inner()
                        .into_iter()
                        .map(|e| { (e.0.clone(), e.1.clone()) })
                        .collect();
                    result.sort_by(|x, y| { match x.0.cmp(&y.0) {
                        std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                        std::cmp::Ordering::Equal => x.1.cmp(&y.1),
                        std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                    } });
                    result
                }),
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

    pub fn get_deps(&self) -> Option<&Vec<Module>> {
        self.dependencies.as_ref()
    }

    pub fn get_sum(&self) -> Option<&Sha256Hash> {
        self.sum.as_ref()
    }

    pub fn get_source(&self) -> &str {
        &self.source.0
    }

    pub fn get_name(&self) -> &PkgId {
        &self.name
    }

    pub fn get_version(&self) -> &Version {
        &self.version
    }

    pub fn to_toml(&self, table: &mut toml_edit::Table) -> () {
        table["name"] = toml_edit::value(&self.name.to_string());
        table["version"] = toml_edit::value(&self.version.to_string());
        if let Some(sum) = self.get_sum() {
            table["sum"] = toml_edit::value(&sum.to_string());
        }
        table["source"] = toml_edit::value(&self.source.0);
        if let Some(deps) = &self.dependencies {
            table.insert("dependencies", toml_edit::Item::Value(toml_edit::Value::Array(Array::new())));
            for entry in deps {
                let mut inline = InlineTable::new();
                // @TODO write newlines after each item?
                inline.insert("name", toml_edit::Value::String(Formatted::new(entry.0.to_string())));
                inline.insert("version", toml_edit::Value::String(Formatted::new(entry.1.to_string())));
                inline.decor_mut().set_prefix("\n    ");
                table["dependencies"].as_array_mut()
                    .unwrap()
                    .push_formatted(toml_edit::Value::InlineTable(inline));
            }
            table["dependencies"].as_array_mut().unwrap().set_trailing("\n");
        }
    }
}

impl FromToml for LockEntry {
    type Err = Fault; 

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            name: PkgId::from_str(table.get("name").unwrap().as_str().unwrap())?,
            version: Version::from_str(table.get("version").unwrap().as_str().unwrap())?,
            sum: match table.get("sum") {
                Some(item) => Some(Sha256Hash::from_str(item.as_str().unwrap())?),
                None => None,
            },
            source: Source(table.get("source").unwrap().as_str().unwrap().to_owned()),
            dependencies: {
                match table.get("dependencies") {
                    Some(item) => {
                        let mut result: Vec<Module> = Vec::new();
                        for entry in item.as_array().unwrap() {
                            let entry = entry.as_inline_table().unwrap();
                            result.push(
                                (
                                    PkgId::from_str(entry["name"].as_str().unwrap()).unwrap(),
                                    AnyVersion::from_str(entry["version"].as_str().unwrap()).unwrap(),
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

pub const IP_LOCK_FILE: &str = "Orbit.lock";