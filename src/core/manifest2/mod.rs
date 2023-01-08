#![allow(dead_code)]

use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use std::path::PathBuf;
use std::fmt::Display;
use super::pkgid::PkgPart;

type Identifier = PkgPart;
type Version = super::version::Version;
type Source = String;


type Deps = Option<Dependencies>;
type DevDeps = Option<Dependencies>;

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    ip: Package,
    dependencies: Deps,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: DevDeps,
}

trait FromFile: FromStr where Self: Sized, <Self as std::str::FromStr>::Err: 'static + std::error::Error {
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
    pub fn new(name: Identifier) -> Self {
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
    pub fn write_empty_manifest(name: Identifier) -> String {
        format!(r#"[ip]
name = "{}"
version = "0.1.0"

# See more keys and their definitions at https://c-rus.github.io/orbit/4_topic/2_orbittoml.html

[dependencies]
"#, name)
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(&self).unwrap())
    }
}

#[derive(Deserialize, Serialize)]
struct Package {
    name: Identifier,
    version: Version,
    library: Option<Identifier>,
    source: Option<Source>,
}

type Dependencies = HashMap<Identifier, Version>;

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
            assert_eq!(man.ip.source, None);
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