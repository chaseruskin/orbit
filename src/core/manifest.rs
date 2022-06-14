use toml_edit::Document;
use std::path;
use std::error::Error;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::AnyError;

pub struct Manifest {
    // track where the file loads/stores from
    path: path::PathBuf, 
    // maintain the data
    document: Document
}

use glob::glob;

/// Finds all Manifest files available in the provided path
pub fn find_dev_manifests(path: &std::path::PathBuf) -> Result<Vec<Manifest>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    // walk the ORBIT_PATH directory @TODO recursively walk directories until hitting first 'Orbit.toml' file.
    for entry in glob(&path.join("**/Orbit.toml").display().to_string()).expect("Failed to read glob pattern") {
        let e = entry?;
        // read ip_spec from each manifest
        result.push(Manifest::load(e)?);
    }
    Ok(result)
}

impl Manifest {
    pub fn create(path: path::PathBuf) -> Self {
        Self {
            path: path,
            document: BARE_MANIFEST.parse::<Document>().unwrap(),
        }
    }

    /// Checks if the manifest has the `ip` table and contains the minimum required keys: `vendor`, `library`,
    /// `name`, `version`.
    pub fn has_bare_min(&self) -> Result<(), AnyError> {
        if self.get_doc().contains_table("ip") == false {
            return Err(AnyError(format!("missing 'ip' table")))
        } else if self.get_doc()["ip"].as_table().unwrap().contains_key("vendor") == false {
            return Err(AnyError(format!("missing required key 'vendor' in table 'ip'")))
        } else if self.get_doc()["ip"].as_table().unwrap().contains_key("library") == false {
            return Err(AnyError(format!("missing required key 'library' in table 'ip'")))
        } else if self.get_doc()["ip"].as_table().unwrap().contains_key("name") == false {
            return Err(AnyError(format!("missing required key 'name' in table 'ip'")))
        } else if self.get_doc()["ip"].as_table().unwrap().contains_key("version") == false {
            return Err(AnyError(format!("missing required key 'version' in table 'ip'")))
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            path: path::PathBuf::new(),
            document: Document::new(),
        }
    }

    /// Loads data from file as a `Manifest` struct. 
    /// 
    /// Errors on parsing errors for toml and errors on any particular rules for
    /// manifest formatting/required keys.
    pub fn load(path: path::PathBuf) -> Result<Self, Box<dyn Error>>{
        // load data from file
        let contents = std::fs::read_to_string(&path)?;
        let m = Self {
            path: path,
            document: contents.parse::<Document>()?,
        };
        // verify bare minimum keys exist for 'ip' table
        match m.has_bare_min() {
            Ok(()) => Ok(m),
            Err(e) => return Err(AnyError(format!("manifest {:?} {}", m.get_path(), e)))?
        }
    }

    /// Stores data to file from `Manifest` struct.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.path, self.document.to_string())?;
        Ok(())
    }

    /// Creates a new `PkgId` from the fields of the manifest document.
    pub fn as_pkgid(&self) -> PkgId {
        PkgId::new().vendor(self.get_doc()["ip"]["vendor"].as_str().unwrap()).unwrap()
            .library(self.get_doc()["ip"]["library"].as_str().unwrap()).unwrap()
            .name(self.get_doc()["ip"]["name"].as_str().unwrap()).unwrap()
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

    #[test]
    fn new() {
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = Manifest::create(m.path().to_path_buf());
        assert_eq!(manifest.document.to_string(), BARE_MANIFEST);
    }

    #[test]
    fn bare_min_valid() {
        // has all keys and 'ip' table
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = Manifest::create(m.path().to_path_buf());
        assert_eq!(manifest.has_bare_min().unwrap(), ());

        // missing all required fields
        let manifest = Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[ip]
".parse::<Document>().unwrap()
        };
        assert_eq!(manifest.has_bare_min().is_err(), true);

        // missing 'version' key
        let manifest = Manifest {
            path: tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
            document: "\
[ip]
vendor = \"v\"
library = \"l\"
name = \"n\"
".parse::<Document>().unwrap()
        };
        assert_eq!(manifest.has_bare_min().is_err(), true);
    }
}