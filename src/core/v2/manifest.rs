#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use std::path::PathBuf;
use std::fmt::Display;
use crate::core::pkgid::PkgPart;
// use crate::util::url::Url;

pub type Id = PkgPart;
pub type Version = crate::core::version::Version;
pub type Source = String;

use crate::core::ip::IpSpec2;
use crate::core::lang::vhdl::primaryunit::PrimaryUnit;
use crate::core::lang::vhdl::token::Identifier;
use crate::util::anyerror::Fault;
use toml_edit::Document;
use crate::util::sha256::Sha256Hash;

type Deps = Option<Dependencies>;
type DevDeps = Option<Dependencies>;

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";
pub const IP_MANIFEST_PATTERN_FILE : &str = "Orbit-*.toml";
pub const ORBIT_SUM_FILE: &str = ".orbit-checksum";
pub const ORBIT_METADATA_FILE: &str = ".orbit-metadata";

const DEPENDENCIES_KEY: &str = "dependencies";

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    ip: Package,
    dependencies: Deps,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: DevDeps,
}

pub trait FromFile: FromStr where Self: Sized, <Self as std::str::FromStr>::Err: 'static + std::error::Error {
    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // try to open the file in read-only mode
        let text = std::fs::read_to_string(&path)?;
        Ok(Self::from_str(&text)?)
    }
}

impl FromFile for Manifest {}

impl FromStr for Manifest {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Manifest {
    /// Creates data for a new [Manifest].
    pub fn new(name: Id) -> Self {
        Self {
            ip: Package {
                name: name,
                version: Version::new().minor(1),
                source: None,
                library: None,
            },
            dependencies: Some(Dependencies::new()),
            dev_dependencies: None,
        }
    }

    /// Composes a [String] to write to a clean manifest file.
    pub fn write_empty_manifest(name: &Id) -> String {
        format!(r#"[ip]
name = "{}"
version = "0.1.0"

# See more keys and their definitions at https://c-rus.github.io/orbit/4_topic/2_orbittoml.html

[dependencies]
"#, name)
    }

    /// Gathers the list of primary design units for the current ip.
    /// 
    /// If the manifest has an toml entry for `units` and `force` is set to `false`, 
    /// then it will return that list rather than go through files.
    pub fn collect_units(force: bool, dir: &PathBuf) -> Result<HashMap<Identifier, PrimaryUnit>, Fault> {
        // try to read from metadata file
        match (force == false) && Self::read_units_from_metadata(&dir).is_some() {
            // use precomputed result
            true => Ok(Self::read_units_from_metadata(&dir).unwrap()),
            false => {
                // collect all files
                let files = crate::util::filesystem::gather_current_files(&dir, false);
                Ok(crate::core::lang::vhdl::primaryunit::collect_units(&files)?)
            }
        }
    }

    pub fn read_units_from_metadata(dir: &PathBuf) -> Option<HashMap<Identifier, PrimaryUnit>> {
        let meta_file = dir.join(ORBIT_METADATA_FILE);
        if std::path::Path::exists(&meta_file) == true {
            if let Ok(contents) = std::fs::read_to_string(&meta_file) {
                if let Ok(toml) = contents.parse::<Document>() {
                    let entry = toml.get("ip")?.as_table()?.get("units")?.as_array()?;
                    let mut map = HashMap::new();
                    for unit in entry {
                        let pdu = PrimaryUnit::from_toml(unit.as_inline_table()?)?;
                        map.insert(pdu.get_iden().clone(), pdu);
                    }
                    Some(map)
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

    pub fn get_ip(&self) -> &Package {
        &self.ip
    }

    pub fn get_deps(&self) -> &Deps {
        &self.dependencies
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Package {
    name: Id,
    version: Version,
    library: Option<Id>,
    /// Describes the URL for fetching the captured state's code (expects .ZIP file)
    source: Option<Source>,
}

impl Package {
    pub fn get_name(&self) -> &Id {
        &self.name
    }

    pub fn get_version(&self) -> &Version {
        &self.version
    }

    pub fn get_library(&self) -> &Option<Id> {
        &self.library
    }

    pub fn get_source(&self) -> &Option<Source> {
        &self.source
    }

    /// Clones into a new [IpSpec2] struct.
    pub fn into_ip_spec(&self) -> IpSpec2 {
        IpSpec2::new(self.get_name().clone(), self.get_version().clone())
    }

    /// Computes the checksum on the root of the IP.
    /// 
    /// Changes the current working directory to the root for consistent computation.
    pub fn compute_checksum(dir: &PathBuf) -> Sha256Hash {
        let ip_files = crate::util::filesystem::gather_current_files(&dir, true);
        let checksum = crate::util::checksum::checksum(&ip_files, &dir);
        checksum
    }

    /// Gets the already calculated checksum from an installed IP from [ORBIT_SUM_FILE].
    /// 
    /// Returns `None` if the file does not exist, is unable to read into a string, or
    /// if the sha cannot be parsed.
    pub fn read_checksum_proof(dir: &PathBuf) -> Option<Sha256Hash> {
        let sum_file = dir.join(ORBIT_SUM_FILE);
        if sum_file.exists() == false {
            None
        } else {
            match std::fs::read_to_string(&sum_file) {
                Ok(text) => {
                    match Sha256Hash::from_str(&text.trim()) {
                        Ok(sha) => Some(sha),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
    }
}

type Dependencies = HashMap<Id, Version>;

#[cfg(test)]
mod test {
    use super::*;

    mod deser {
        use super::*;

        #[test]
        fn ut_minimal() {
            let man: Manifest = toml::from_str(EX2).unwrap();
    
            assert_eq!(man.ip.name, PkgPart::from_str("Lab1").unwrap());
            assert_eq!(man.ip.version, Version::new().major(1));
            assert_eq!(man.ip.source, None);
            assert_eq!(man.dependencies, None);
            assert_eq!(man.dev_dependencies, None);
        }

        #[test]
        fn ut_complex() {
            let man: Manifest = toml::from_str(EX1).unwrap();
    
            assert_eq!(man.ip.name, PkgPart::from_str("gates").unwrap());
            assert_eq!(man.ip.source, Some(String::from("https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip")));
            assert_eq!(man.dependencies.unwrap().len(), 1);
            assert_eq!(man.dev_dependencies.unwrap().len(), 2);
            assert_eq!(man.ip.library, Some(PkgPart::from_str("common").unwrap()));
        }

        #[test]
        fn ut_bad() {
            let man = toml::from_str::<Manifest>(ERR1);

            assert_eq!(man.is_err(), true);
        }

        #[test]
        fn ut_serialize() {
            // @note: keys in an table-array (hashmap) are not guaranteed to be in the same order
            let man: Manifest = toml::from_str(EX3).unwrap();
            let text = toml::to_string(&man).unwrap();
            assert_eq!(text, EX3);
        }
    }


    #[test]
    fn compute_checksum() {
        let sum = Package::compute_checksum(&PathBuf::from("./tests/env/project1/"));
        assert_eq!(sum, Sha256Hash::from_u32s([
            2472527351, 1678808787, 3321465315, 1927515725, 
            108238780, 2368649324, 2487325306, 4053483655]))
    }
}


const EX1: &str = r#"[ip]
name = "gates"
version = "0.1.0"
library = "common"
source = "https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip"

[dependencies]
some-package = "10.0.0"

[dev-dependencies]
top-builder = "1.0.0"
my-testing-framework = "0.1.0"
"#;

const EX2: &str = r#"[ip]
name = "Lab1"
version = "1.0.0"
"#;

const EX3: &str = r#"[ip]
name = "lab2"
version = "1.20.0"

[dependencies]
some-package = "9.0.0"

[dev-dependencies]
top-builder = "1.0.0"
"#;

const ERR1: &str = r#"[ip]
"#;