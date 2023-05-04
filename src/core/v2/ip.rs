use std::path::PathBuf;
use crate::core::v2::manifest::Manifest;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::core::v2::manifest;

use super::manifest::FromFile;
use crate::core::v2::manifest::ORBIT_METADATA_FILE;
use crate::core::v2::manifest::IP_MANIFEST_FILE;
use crate::core::v2::manifest::ORBIT_SUM_FILE;
use toml_edit::Document;
use crate::util::sha256::Sha256Hash;
use crate::core::lang::vhdl::primaryunit::PrimaryUnit;
use crate::core::lang::vhdl::token::Identifier;
use std::str::FromStr;
use std::collections::HashMap;

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

    /// Checks if the given path hosts a valid manifest file.
    pub fn is_valid(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let man_path = path.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(AnyError(format!("A manifest file does not exist")))?
        }
        // attempt to load the manifest file
        let _ = Manifest::from_file(&man_path)?;
        return Ok(())
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

    /// Caches the result of collecting all the primary design units for the given package.
    /// 
    /// Writes the data to the toml data structure. Note, this function does not save the manifest data to file.
    // pub fn stash_units(&mut self) -> () {
    //     // collect the units
    //     let units = Self::collect_units(true).unwrap();
    //     let tbl = self.get_manifest_mut().get_mut_doc()["ip"].as_table_mut().unwrap();
    //     tbl.insert("units", toml_edit::Item::Value(toml_edit::Value::Array(Array::new())));
    //     let arr = tbl["units"].as_array_mut().unwrap();
    //     // map the units into a serialized data format
    //     for (_, unit) in &units {
    //         arr.push(unit.to_toml());
    //     }
    //     tbl["units"].as_array_mut().unwrap().iter_mut().for_each(|f| {
    //         f.decor_mut().set_prefix("\n    ");
    //         f.decor_mut().set_suffix("");
    //     });
    //     tbl["units"].as_array_mut().unwrap().set_trailing("\n");
    // }

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

    // /// Adds to manifest file to set as dynamic.
    // pub fn set_as_dynamic(&mut self) -> () {
    //     self.data.get_mut_doc().as_table_mut()["dynamic"] = value(true);
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compute_checksum() {
        let sum = Ip::compute_checksum(&PathBuf::from("./tests/env/project1/"));
        assert_eq!(sum, Sha256Hash::from_u32s([
            2472527351, 1678808787, 3321465315, 1927515725, 
            108238780, 2368649324, 2487325306, 4053483655]))
    }
}