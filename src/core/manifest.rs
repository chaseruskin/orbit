#![allow(dead_code)]

use crate::core::ip::IpSpec;
use crate::core::pkgid::PkgPart;
use crate::core::source;
use crate::core::source::Source;
use crate::util::anyerror::{AnyError, Fault};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;
use std::{collections::HashMap, str::FromStr};

use super::lang::vhdl::token::identifier::Identifier;

pub type Id = PkgPart;
pub type Version = crate::core::version::Version;

type Dependencies = HashMap<Id, Version>;

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";
// pub const IP_MANIFEST_PATTERN_FILE : &str = "Orbit-*.toml";
pub const ORBIT_SUM_FILE: &str = ".orbit-checksum";
pub const ORBIT_METADATA_FILE: &str = ".orbit-metadata";

const DEPENDENCIES_KEY: &str = "dependencies";

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    ip: Package,
    #[serde(skip_serializing_if = "map_is_empty", default)]
    dependencies: Dependencies,
    #[serde(
        rename = "dev-dependencies",
        skip_serializing_if = "map_is_empty",
        default
    )]
    dev_dependencies: Dependencies,
}

pub trait FromFile: FromStr
where
    Self: Sized,
    <Self as std::str::FromStr>::Err: 'static + Error,
{
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
        let man = match Self::from_str(&contents) {
            Ok(r) => r,
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!(
                    "failed to parse {} file: {}",
                    IP_MANIFEST_FILE, e
                )))?
            }
        };
        // verify there are no duplicate entries between tables
        if let Some(e) = man.is_deps_valid().err() {
            return Err(AnyError(format!(
                "failed to parse {} file: {}",
                IP_MANIFEST_FILE, e
            )))?;
        }
        Ok(man)
    }
}

impl FromStr for Manifest {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Manifest {
    /// Establishes a minimal bare [Manifest].
    pub fn new() -> Self {
        Self {
            ip: Package {
                name: PkgPart::new(),
                version: Version::new(),
                source: None.into(),
                keywords: Vec::new(),
                summary: None,
                library: None,
                readme: None,
                authors: None,
                metadata: HashMap::new(),
            },
            dependencies: Dependencies::new(),
            dev_dependencies: Dependencies::new(),
        }
    }

    /// Returns the library name to be used for HDL. If a library is not specified,
    /// then it chooses "work".
    pub fn get_hdl_library(&self) -> Identifier {
        match self.get_ip().get_library().as_ref() {
            Some(l) => Identifier::from(l),
            None => Identifier::new_working(), // Identifier::from(self.get_man().get_ip().get_name()),
        }
    }

    /// Composes a [String] to write to a clean manifest file.
    pub fn write_empty_manifest(name: &Id) -> String {
        format!(
            r#"[ip]
name = "{}"
version = "0.1.0"

# See more keys and their definitions at https://cdotrus.github.io/orbit/reference/manifest.html

[dependencies]
"#,
            name
        )
    }

    pub fn get_ip(&self) -> &Package {
        &self.ip
    }

    /// Returns the list of dependencies found only under the "dependencies"
    /// table.
    pub fn get_deps(&self) -> &Dependencies {
        &self.dependencies
    }

    pub fn get_dev_deps(&self) -> &Dependencies {
        &self.dev_dependencies
    }

    pub fn is_deps_valid(&self) -> Result<(), AnyError> {
        for (key, _) in &self.dependencies {
            if let Some(_) = self.dev_dependencies.get(key) {
                return Err(AnyError(format!(
                    "duplicate key '{}' in [dependencies] and [dev-dependencies]",
                    key
                )));
                // println!("{}: Dependency {} is used instead of dev-dependency {}",
                //     "warning".yellow().bold(),
                //     IpSpec::from((key.clone(), val.clone())),
                //     IpSpec::from((key.clone(), rep.clone())),
                // )
            }
        }
        Ok(())
    }

    /// Returns the list of dependencies found under "dependencies" and
    /// "dev-dependencies".
    pub fn get_deps_list(&self, include_dev: bool) -> Vec<(&PkgPart, &Version)> {
        let mut result = Vec::with_capacity(
            self.dependencies.len()
                + match include_dev {
                    true => self.dev_dependencies.len(),
                    false => 0,
                },
        );
        if include_dev == true {
            result.extend(self.dev_dependencies.iter());
        }
        result.extend(self.dependencies.iter());
        result
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

fn vec_is_empty<T>(field: &Vec<T>) -> bool {
    field.is_empty()
}

fn map_is_empty<K, V>(field: &HashMap<K, V>) -> bool {
    field.is_empty()
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Package {
    name: Id,
    version: Version,
    authors: Option<Vec<String>>,
    summary: Option<String>,
    library: Option<Id>,
    #[serde(skip_serializing_if = "vec_is_empty", default)]
    keywords: Vec<String>,
    /// Describes the URL for fetching the captured state's code (expects .ZIP file)
    #[serde(deserialize_with = "source::string_or_struct", default)]
    source: Source,
    readme: Option<PathBuf>,
    /// Ignore this field and never use it for any processing
    #[serde(skip_serializing_if = "map_is_empty", default)]
    metadata: HashMap<String, toml::Value>,
}

impl Package {
    pub fn get_name(&self) -> &Id {
        &self.name
    }

    pub fn get_version(&self) -> &Version {
        &self.version
    }

    pub fn get_keywords(&self) -> &Vec<String> {
        &self.keywords
    }

    pub fn get_library(&self) -> &Option<Id> {
        &self.library
    }

    pub fn get_source(&self) -> Option<&Source> {
        self.source.as_option()
    }

    /// Clones into a new [IpSpec2] struct.
    pub fn into_ip_spec(&self) -> IpSpec {
        IpSpec::new(self.get_name().clone(), self.get_version().clone())
    }

    pub fn get_readme(&self) -> &Option<PathBuf> {
        &self.readme
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
        return Ok(vec![path.to_path_buf()]);
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
            assert_eq!(man.ip.get_source(), None);
            assert_eq!(man.dependencies, HashMap::new());
            assert_eq!(man.dev_dependencies, HashMap::new());
        }

        #[test]
        fn ut_complex() {
            let man: Manifest = toml::from_str(EX1).unwrap();

            assert_eq!(man.ip.name, PkgPart::from_str("gates").unwrap());
            assert_eq!(
                man.ip.get_source(),
                Some(
                    &Source::from_str(
                        "https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip"
                    )
                    .unwrap()
                )
            );
            assert_eq!(man.dependencies.len(), 1);
            assert_eq!(man.dev_dependencies.len(), 2);
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
            let text = man.to_string();
            assert_eq!(text, EX3);
        }

        #[test]
        fn ut_complex_source() {
            let man: Manifest = match toml::from_str(EX4) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            println!("{}", toml::to_string(&man).unwrap());

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .unwrap(),
                "ktsp"
            );

            let man: Manifest = match toml::from_str(EX5) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .as_ref(),
                None
            );

            let man: Manifest = match toml::from_str(EX6) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };

            assert_eq!(man.ip.get_source().is_some(), true);
            assert_eq!(
                man.ip.get_source().as_ref().unwrap().get_url(),
                "https://some.url"
            );
            assert_eq!(
                man.ip
                    .get_source()
                    .as_ref()
                    .unwrap()
                    .get_protocol()
                    .as_ref(),
                None
            );
        }

        #[test]
        #[should_panic]
        fn ut_source_missing_url() {
            // missing required key "url"
            let _man: Manifest = match toml::from_str(EX7) {
                Ok(m) => m,
                Err(e) => panic!("{}", e.to_string()),
            };
        }
    }
}

const EX1: &str = r#"[ip]
name = "gates"
version = "0.1.0"
library = "common"
source = "https://github.com/ks-tech/gates/archive/refs/tags/0.1.0.zip"

[ip.metadata]
foo = 1
bar = 2

[ip.metadata.subtable]
foo = "hello world"

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
keywords = ["circuits"]

[dependencies]
some-package = "9.0.0"

[dev-dependencies]
top-builder = "1.0.0"
"#;

const EX4: &str = r#"[ip]
name = "lab2"
version = "1.20.0"
source = { url = "https://some.url", protocol = "ktsp" }
"#;

const EX5: &str = r#"[ip]
name = "lab2"
version = "1.20.0"
source = { url = "https://some.url" }
"#;

const EX6: &str = r#"[ip]
name = "lab2"
version = "1.20.0"
source = "https://some.url"
"#;

const EX7: &str = r#"[ip]
name = "lab2"
version = "1.20.0"
source = { protocol = "ktsp" }
"#;

const ERR1: &str = r#"[ip]
"#;
