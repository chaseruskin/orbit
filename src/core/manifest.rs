use toml_edit::Document;
use std::path;
use std::error::Error;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::AnyError;
use std::path::PathBuf;
use glob::glob;
use std::str::FromStr;

#[derive(Debug)]
pub struct Manifest {
    // track where the file loads/stores from
    path: path::PathBuf, 
    // maintain the data
    document: Document
}

impl Manifest {
    /// Finds all Manifest files available in the provided path `path`.
    /// 
    /// Errors if on filesystem problems.
    pub fn detect_all(path: &std::path::PathBuf, name: &str) -> Result<Vec<Manifest>, Box<dyn std::error::Error>> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file.
        for entry in glob(&path.join(format!("**/{}", name)).display().to_string()).expect("Failed to read glob pattern") {
            let e = entry?;
            // read ip_spec from each manifest
            result.push(Manifest::from_path(e)?);
        }
        Ok(result)
    }

    /// Reads from the file at `path` and parses into a valid toml document for a `Manifest` struct. 
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            // load the data as a string
            document: std::fs::read_to_string(&path)?.parse::<Document>()?,
            path: path,     
        })
    }

    /// Edits the .toml document at the `table`.`key` with `value`.
    /// 
    pub fn write<T>(&mut self, table: &str, key: &str, value: T) -> ()
    where toml_edit::Value: From<T> {
        self.document[table][key] = toml_edit::value(value);
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

    /// Creates a new empty `Manifest` struct.
    pub fn new() -> Self {
        Self {
            path: path::PathBuf::new(),
            document: Document::new(),
        }
    }

    /// Stores data to file from `Manifest` struct.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.path, self.document.to_string())?;
        Ok(())
    }

    pub fn get_doc(&self) -> &Document {
        &self.document
    }

    pub fn get_path(&self) -> &path::PathBuf {
        &self.path
    }

    pub fn get_mut_doc(&mut self) -> &mut Document {
        &mut self.document
    }
}

pub const IP_MANIFEST_FILE: &str = "Orbit.toml";

#[derive(Debug)]
pub struct IpManifest(pub Manifest);

impl IpManifest {
    /// Creates an empty `IpManifest` struct.
    pub fn new() -> Self {
        IpManifest(Manifest::new())
    }

    /// Finds all IP manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Ok(Manifest::detect_all(path, IP_MANIFEST_FILE)?.into_iter().map(|f| IpManifest(f)).collect())
    }

    /// Creates a new minimal IP manifest for `path`.
    /// 
    /// Does not actually write the data to `path`. Use the `fn save` to write to disk.
    pub fn init(path: path::PathBuf) -> Self {
        Self(Manifest {
            path: path,
            document: BARE_MANIFEST.parse::<Document>().unwrap(),
        })
    }

    /// Creates a new `PkgId` from the fields of the manifest document.
    /// 
    /// Assumes the manifest document contains a table 'ip' with the necessary keys.
    pub fn as_pkgid(&self) -> PkgId {
        PkgId::new().vendor(self.0.get_doc()["ip"]["vendor"].as_str().unwrap()).unwrap()
            .library(self.0.get_doc()["ip"]["library"].as_str().unwrap()).unwrap()
            .name(self.0.get_doc()["ip"]["name"].as_str().unwrap()).unwrap()
    }

    /// Loads data from file as a `Manifest` struct. 
    /// 
    /// Errors on parsing errors for toml and errors on any particular rules for
    /// manifest formatting/required keys.
    fn from_manifest(m: Manifest) -> Result<Self, Box<dyn Error>> {
        let ip = IpManifest(m);
        // verify bare minimum keys exist for 'ip' table
        match ip.has_bare_min() {
            Ok(()) => Ok(ip),
            Err(e) => return Err(AnyError(format!("manifest {:?} {}", ip.0.get_path(), e)))?
        }
    }

    /// Loads an `IpManifest` from `path`.
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        Ok(Self(Manifest::from_path(path)?))
    }

    /// Checks if the manifest has the `ip` table and contains the minimum required keys: `vendor`, `library`,
    /// `name`, `version`.
    pub fn has_bare_min(&self) -> Result<(), AnyError> {
        if self.0.get_doc().contains_table("ip") == false {
            return Err(AnyError(format!("missing 'ip' table")))
        } else if self.0.get_doc()["ip"].as_table().unwrap().contains_key("vendor") == false {
            return Err(AnyError(format!("missing required key 'vendor' in table 'ip'")))
        } else if self.0.get_doc()["ip"].as_table().unwrap().contains_key("library") == false {
            return Err(AnyError(format!("missing required key 'library' in table 'ip'")))
        } else if self.0.get_doc()["ip"].as_table().unwrap().contains_key("name") == false {
            return Err(AnyError(format!("missing required key 'name' in table 'ip'")))
        } else if self.0.get_doc()["ip"].as_table().unwrap().contains_key("version") == false {
            return Err(AnyError(format!("missing required key 'version' in table 'ip'")))
        }
        Ok(())
    }

    /// Collects all direct dependency IP from the `[dependencies]` table.
    /// 
    /// Errors if there is an invalid entry in the table.
    pub fn get_dependencies(&self) -> Result<Vec<PkgId>, Box<dyn std::error::Error>> {
        let mut deps = Vec::new();
        // check if the table exists and return early if does not
        if self.0.get_doc().contains_table("dependencies") == false {
            return Ok(deps)
        }
        // traverse three tables deep to retrieve V.L.N
        for v in self.0.get_doc().get("dependencies").unwrap().as_table().unwrap() {
            for l in v.1.as_table().unwrap() {
                for n in l.1.as_table().unwrap() {
                    deps.push(PkgId::from_str(&format!("{}.{}.{}", v.0, l.0, n.0))?);
                }
            }
        }
        Ok(deps)
    }

    /// Gets the remote repository value, if any.
    pub fn get_repository(&self) -> Option<String> {
        self.0.read_as_str("ip", "repository")
    }
}

const BARE_MANIFEST: &str = "\
[ip]
name    = \"\"
library = \"\"
version = \"0.1.0\"
vendor  = \"\"

# To learn more about writing the manifest, see https://github.com/c-rus/orbit

[dependencies]
";

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn new() {
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = IpManifest::init(m.path().to_path_buf());
        assert_eq!(manifest.0.document.to_string(), BARE_MANIFEST);
    }

    #[test]
    fn bare_min_valid() {
        // has all keys and 'ip' table
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = IpManifest::init(m.path().to_path_buf());
        assert_eq!(manifest.has_bare_min().unwrap(), ());

        // missing all required fields
        let manifest = IpManifest(Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[ip]
".parse::<Document>().unwrap()
        });
        assert_eq!(manifest.has_bare_min().is_err(), true);

        // missing 'version' key
        let manifest = IpManifest(Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[ip]
vendor = \"v\"
library = \"l\"
name = \"n\"
".parse::<Document>().unwrap()
        });
        assert_eq!(manifest.has_bare_min().is_err(), true);
    }

    #[test]
    fn get_deps() {
        // empty table
        let manifest = IpManifest(Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[dependencies]
".parse::<Document>().unwrap()
        });
        assert_eq!(manifest.get_dependencies().unwrap(), vec![]);

        // no `dependencies` table
        let manifest = IpManifest(Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[ip]
name = \"gates\"
".parse::<Document>().unwrap()
        });
        assert_eq!(manifest.get_dependencies().unwrap(), vec![]);

        // `dependencies` table with entries
        let manifest = IpManifest(Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[dependencies]
ks_tech.rary.gates = \"1.0.0\"
ks_tech.util.toolbox = \"2\"
c_rus.eel4712c.lab1 = \"4.2\"
".parse::<Document>().unwrap()
        });
        assert_eq!(manifest.get_dependencies().unwrap(), vec![
            PkgId::from_str("ks_tech.rary.gates").unwrap(),
            PkgId::from_str("ks_tech.util.toolbox").unwrap(),
            PkgId::from_str("c_rus.eel4712c.lab1").unwrap(),
        ]);
    }


    mod vendor {
        use super::*;
        use crate::core::vendor::VendorManifest;
        use std::str::FromStr;
        
        #[test]
        fn read_index() {
            let doc = "\
[vendor]
name = \"ks-tech\"

[index]
rary.gates = \"url1\"
memory.ram = \"url2\"
    ";
            let manifest = VendorManifest(Manifest {
                path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
                document: doc.parse::<Document>().unwrap()
            });

            assert_eq!(manifest.read_index(), vec![
                PkgId::from_str("ks-tech.rary.gates").unwrap(), 
                PkgId::from_str("ks-tech.memory.ram").unwrap()
            ]);
        }
    }
}