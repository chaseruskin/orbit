#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use std::path::PathBuf;
use std::fmt::Display;
use crate::core::pkgid::PkgPart;
use std::error::Error;
// use crate::util::url::Url;

pub type Id = PkgPart;
pub type Version = crate::core::version::Version;
pub type Source = String;
use crate::core::v2::ip::IpSpec;
use crate::util::anyerror::{Fault, AnyError};

type Dependencies = HashMap<Id, Version>;

type Deps = Option<Dependencies>;
type DevDeps = Option<Dependencies>;

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";
pub const IP_MANIFEST_PATTERN_FILE : &str = "Orbit-*.toml";
pub const ORBIT_SUM_FILE: &str = ".orbit-checksum";
pub const ORBIT_METADATA_FILE: &str = ".orbit-metadata";

const DEPENDENCIES_KEY: &str = "dependencies";

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Manifest {
    ip: Package,
    dependencies: Deps,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: DevDeps,
}

pub trait FromFile: FromStr where Self: Sized, <Self as std::str::FromStr>::Err: 'static + Error {
    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // try to open the file in read-only mode
        let text = std::fs::read_to_string(&path)?;
        Ok(Self::from_str(&text)?)
    }
}

impl FromFile for Manifest {

    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(r) => Ok(r),
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!("failed to parse {} file: {}", IP_MANIFEST_FILE, e)))?
            }
        }
    }
}

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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
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
    pub fn into_ip_spec(&self) -> IpSpec {
        IpSpec::new(self.get_name().clone(), self.get_version().clone())
    }
}

/// Takes an iterative approach to iterating through directories to find a file
/// matching `name`.
/// 
/// Note: `name` is become a glob-style pattern.
/// 
/// Stops descending the directories upon finding first match of `name`. 
/// The match must be case-sensitive. If `is_exclusive` is `false`, then the directory
/// with match will continued to be searched at that level and then re-track.
pub fn find_file(path: &PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<PathBuf>, Fault> {
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