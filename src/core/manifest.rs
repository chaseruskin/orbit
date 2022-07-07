use toml_edit::{Document};
use std::collections::HashMap;
use std::path;
use std::path::PathBuf;
use std::error::Error;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::{AnyError, Fault};
use std::str::FromStr;
use crate::core::version::Version;

use super::config::{FromToml, FromTomlError};
use super::version::AnyVersion;

#[derive(Debug)]
pub struct Manifest {
    // track where the file loads/stores from
    path: path::PathBuf, 
    // maintain the data
    document: Document
}

/// Takes an iterative approach to iterating through directories to find a file
/// matching `name`.
/// 
/// Stops descending the directories upon finding first match of `name`. The match
/// must be case-sensitive.
fn find_file(path: &PathBuf, name: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    // list of directories to continue to process
    let mut to_process: Vec<PathBuf> = Vec::new();
    let mut result = Vec::new();
    // start at base path
    if path.is_dir() {
        to_process.push(path.to_path_buf());
    // only look at file and exit
    } else if path.is_file() && path.file_name().unwrap() == name {
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
                if e.file_name().as_os_str() == name {
                    result.push(e.path());
                    found_file = true;
                    break;
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

impl Manifest {
    /// Finds all Manifest files available in the provided path `path`.
    /// 
    /// Errors if on filesystem problems.
    pub fn detect_all(path: &std::path::PathBuf, name: &str) -> Result<Vec<Manifest>, Box<dyn std::error::Error>> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file
        for entry in find_file(&path, &name)? {
            // read ip_spec from each manifest
            result.push(Manifest::from_path(entry)?);
        }
        Ok(result)
    }

    /// Reads from the file at `path` and parses into a valid toml document for a `Manifest` struct. 
    /// 
    /// Errors if the file does not exist or the TOML parsing fails.
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if std::path::Path::exists(&path) == false {
            return Err(AnyError(format!("missing manifest file {:?}", path)))?
        }
        Ok(Self {
            // load the data as a string
            document: std::fs::read_to_string(&path)?.parse::<Document>()?,
            path: path,     
        })
    }

    /// Edits the .toml document at the `table`.`key` with `value`.
    /// 
    pub fn write<T: ToString>(&mut self, table: &str, key: &str, value: T) -> ()
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
pub struct IpManifest{ 
    manifest: Manifest,
    ip: IpToml,
}

#[derive(Debug)]
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
        write!(f, "\
ip:         {}
summary:    {}
version:    {}
repository: {}
size:       {:.2} MB", 
self.get_pkgid(), 
self.get_summary().unwrap_or(&"".to_string()), 
self.get_version(),
self.get_repository().unwrap_or(&"".to_string()),
crate::util::filesystem::compute_size(&self.manifest.get_path().parent().unwrap(), crate::util::filesystem::Unit::MegaBytes).unwrap()
    )}
}

#[derive(Debug, PartialEq)]
pub struct Ip {
    name: PkgId,
    version: Version,
    repository: Option<String>,
    summary: Option<String>,
    changelog: Option<String>,
    readme: Option<String>,
}

impl Ip {
    pub fn new() -> Self {
        Self { 
            name: PkgId::new(), 
            version: Version::new(), 
            repository: None, 
            summary: None, 
            changelog: None, 
            readme: None
        }
    }

    pub fn get_repository(&self) -> Option<&String> {
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
                None => panic!("expects `ip` to be a toml table")
            }
        } else {
            panic!("missing table `ip`")
        };
        // grab the dependencies table
        let dt = if let Some(item) = table.get("dependencies") {
            match item.as_table() {
                Some(tbl) => DependencyTable::from_toml(tbl)?,
                None => panic!("expects `dependencies` to be a toml table")
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

impl DependencyTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl FromToml for DependencyTable {
    type Err = Fault;
    
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let mut map = HashMap::new();
        // traverse three tables deep to retrieve V.L.N
        for (vendor, v_item) in table.iter() {
            for (library, l_item) in v_item.as_table().unwrap() {
                for (name, n_item) in l_item.as_table().unwrap() {
                    let pkgid = PkgId::new().name(name)?.library(library)?.vendor(vendor)?; 
                    // create version
                    let version = match n_item.as_str() {
                        Some(s) => AnyVersion::from_str(s)?,
                        None => return Err(FromTomlError::ExpectingString(format!("{}.{}.{}", vendor, library, name)))?
                    };
                    // insert into lut
                    map.insert(pkgid, version);
                }
            }
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

    /// Finds all IP manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Manifest::detect_all(path, IP_MANIFEST_FILE)?.into_iter().map(|f| IpManifest::from_manifest(f)).collect()
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
        Ok(IpManifest { ip: IpToml::from_toml(&m.get_doc().as_table())?, manifest: m, })
    }

    /// Loads an `IpManifest` from `path`.
    pub fn from_path(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let man = Manifest::from_path(path)?;
        Ok(Self {
            ip: IpToml::from_toml(man.get_doc().as_table())?,
            manifest: man,
        })
    }

    pub fn get_dependencies(&self) -> &DependencyTable {
        &self.ip.deps
    }

    pub fn get_repository(&self) -> Option<&String> {
        self.ip.ip.get_repository()
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