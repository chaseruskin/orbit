use std::path::PathBuf;
use crate::core::v2::manifest::Manifest;
use crate::util::anyerror::Fault;
use crate::core::v2::manifest;

use super::manifest::FromFile;
use crate::core::v2::manifest::ORBIT_METADATA_FILE;
use crate::core::v2::manifest::IP_MANIFEST_FILE;
use toml_edit::Document;

#[derive(Debug, PartialEq)]
pub struct Ip {
    root: PathBuf,
    data: Manifest,
}

impl Ip {
    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    pub fn get_man(&self) -> &Manifest {
        &self.data
    }

    /// Finds all Manifest files available in the provided path `path`.
    /// 
    /// Errors if on filesystem problems.
    fn detect_all_sub(path: &PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<Self>, Fault> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file
        for entry in manifest::find_file(&path, &name, is_exclusive)? {
            // read ip_spec from each manifest
            let man = Manifest::from_file(&entry)?;
            result.push( Self {
                root: entry, 
                data: man,
            });
        }
        Ok(result)
    }

    /// Finds all IP manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Self::detect_all_sub(path, IP_MANIFEST_FILE, true)
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

    // /// Adds to manifest file to set as dynamic.
    // pub fn set_as_dynamic(&mut self) -> () {
    //     self.data.get_mut_doc().as_table_mut()["dynamic"] = value(true);
    // }
}
