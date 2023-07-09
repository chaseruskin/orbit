use crate::core::manifest;
use crate::core::manifest::Manifest;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use std::path::PathBuf;

use super::iparchive::IpArchive;
use super::lockfile::LockFile;
use super::lockfile::IP_LOCK_FILE;
use super::manifest::FromFile;
use crate::core::lang::vhdl::primaryunit::PrimaryUnit;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::lockfile::LockEntry;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::manifest::ORBIT_METADATA_FILE;
use crate::core::manifest::ORBIT_SUM_FILE;
use crate::core::uuid::Uuid;
use crate::util::sha256::Sha256Hash;
use colored::Colorize;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use toml_edit::Document;

// add state to `root` (make enum) to determine if is real path or not
pub enum IpLocation {
    Physical(PathBuf),
    Virtual,
}

#[derive(Debug, PartialEq)]
pub struct Ip {
    /// The base directory for the entire [Ip] structure.
    root: PathBuf,
    /// The metadata for the [Ip].
    data: Manifest,
    /// The lockfile for the [Ip].
    lock: LockFile,
    /// The UUID for the [Ip].
    uuid: Uuid,
}

impl From<IpArchive> for Ip {
    fn from(value: IpArchive) -> Self {
        let (man, lock) = value.decouple();
        let uuid = match lock.get(man.get_ip().get_name(), man.get_ip().get_version()) {
            Some(entry) => entry.get_uuid().clone(),
            None => Uuid::new(),
        }; 
        Self {
            root: PathBuf::new(),
            data: man,
            lock: lock,
            uuid: uuid,
        }
    }
}

impl Ip {
    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    pub fn get_man(&self) -> &Manifest {
        &self.data
    }

    pub fn get_lock(&self) -> &LockFile {
        &self.lock
    }

    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn load(root: PathBuf) -> Result<Self, Box<dyn Error>> {
        let man_path = root.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(AnyError(format!("A manifest file does not exist")))?;
        }
        let man = Manifest::from_file(&man_path)?;

        let lock_path = root.join(IP_LOCK_FILE);

        let lock = match LockFile::from_file(&lock_path) {
            Ok(l) => l,
            Err(e) => {
                println!(
                    "{}: failed to parse {} file: {}",
                    "warning".yellow().bold(),
                    IP_LOCK_FILE,
                    e
                );
                LockFile::new()
            }
        };

        let uuid = match lock.get(man.get_ip().get_name(), man.get_ip().get_version()) {
            Some(entry) => entry.get_uuid().clone(),
            None => Uuid::new(),
        };

        Ok(Self {
            root: root,
            data: man,
            lock: lock,
            uuid: uuid,
        })
    }

    /// Checks if the given path hosts a valid manifest file.
    pub fn is_valid(path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let man_path = path.join(IP_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(AnyError(format!("A manifest file does not exist")))?;
        }
        // attempt to load the manifest file
        let _ = Manifest::from_file(&man_path)?;
        return Ok(());
    }

    /// Finds all Manifest files available in the provided path `path`.
    ///
    /// Errors if on filesystem problems.
    fn detect_all_sub(path: &PathBuf, name: &str, is_exclusive: bool) -> Result<Vec<Self>, Fault> {
        let mut result = Vec::new();
        // walk the ORBIT_PATH directory @TODO recursively walk inner directories until hitting first 'Orbit.toml' file
        for mut entry in manifest::find_file(&path, &name, is_exclusive)? {
            // remove the manifest file to access the ip's root directory
            entry.pop();
            result.push(Ip::load(entry)?);
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
        self.get_root().join(".orbit-dynamic").exists() == true
    }

    pub fn generate_dst_lut(&self) -> HashMap<Identifier, String> {
        // @todo: read units from metadata to speed up results
        let units = Self::collect_units(true, self.get_root()).unwrap();
        let checksum = Ip::read_checksum_proof(self.get_root()).unwrap();
        // compose the lut for symbol transformation
        let mut lut = HashMap::new();
        units.into_iter().for_each(|(key, _)| {
            lut.insert(
                key.clone(),
                "_".to_string() + checksum.to_string().get(0..10).unwrap(),
            );
        });
        lut
    }

    pub fn set_as_dynamic(&self) -> () {
        let _ = std::fs::write(self.get_root().join(".orbit-dynamic"), "").unwrap();
    }

    /// Checks if needing to read off the lock file.
    ///
    /// This determines if the lock file's data matches the Orbit.toml manifest data,
    /// indicating it is safe to pull data from the lock file and no changes would be
    /// made to the lock file.
    pub fn can_use_lock(&self) -> bool {
        let target = self.get_lock().get(
            self.get_man().get_ip().get_name(),
            self.get_man().get_ip().get_version(),
        );
        match target {
            Some(entry) => entry.matches_target(&LockEntry::from((self, true))),
            None => false,
        }
    }

    /// Checks if the lockfile exists
    pub fn lock_exists(&self) -> bool {
        self.lock.is_empty() == false
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
                Ok(text) => match Sha256Hash::from_str(&text.trim()) {
                    Ok(sha) => Some(sha),
                    Err(_) => None,
                },
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
    pub fn collect_units(
        force: bool,
        dir: &PathBuf,
    ) -> Result<HashMap<Identifier, PrimaryUnit>, Fault> {
        // try to read from metadata file
        match (force == false) && Self::read_units_from_metadata(&dir).is_some() {
            // use precomputed result
            true => Ok(Self::read_units_from_metadata(&dir).unwrap()),
            false => {
                // collect all files
                let files = filesystem::gather_current_files(&dir, false);
                Ok(primaryunit::collect_units(&files)?)
            }
        }
    }

    pub fn read_units_from_metadata(dir: &PathBuf) -> Option<HashMap<Identifier, PrimaryUnit>> {
        let meta_file: PathBuf = dir.join(ORBIT_METADATA_FILE);
        if Path::exists(&meta_file) == true {
            if let Ok(contents) = fs::read_to_string(&meta_file) {
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
}

use crate::core::lang::vhdl::primaryunit;
use crate::core::pkgid::PkgPart;
use crate::core::version::Version;
use crate::util::filesystem;
use std::fs;
use std::path::Path;

const SPEC_DELIM: &str = ":";

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct IpSpec(PkgPart, Version);

impl IpSpec {
    pub fn new(id: PkgPart, version: Version) -> Self {
        Self(id, version)
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.0
    }

    pub fn get_version(&self) -> &Version {
        &self.1
    }
}

impl FromStr for IpSpec {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split by delimiter
        match s.rsplit_once(SPEC_DELIM) {
            Some((n, v)) => Ok(Self::new(PkgPart::from_str(n)?, Version::from_str(v)?)),
            None => Err(Box::new(AnyError(format!(
                "missing specification delimiter {}",
                SPEC_DELIM
            )))),
        }
    }
}

impl std::fmt::Display for IpSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.get_name(), SPEC_DELIM, self.get_version())
    }
}

impl From<(PkgPart, Version)> for IpSpec {
    fn from(value: (PkgPart, Version)) -> Self {
        Self(value.0, value.1)
    }
}

use serde::de::{self};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

impl<'de> Deserialize<'de> for IpSpec {
    fn deserialize<D>(deserializer: D) -> Result<IpSpec, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = IpSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an identifier and a version")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match IpSpec::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for IpSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

use crate::core::version::AnyVersion;

#[derive(Debug, PartialEq)]
pub struct PartialIpSpec(PkgPart, AnyVersion);

impl PartialIpSpec {
    pub fn new() -> Self {
        Self(PkgPart::new(), AnyVersion::Latest)
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.0
    }

    pub fn get_version(&self) -> &AnyVersion {
        &self.1
    }
}

impl FromStr for PartialIpSpec {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.rsplit_once(SPEC_DELIM) {
            // split by delimiter (beginning from rhs)
            Some((n, v)) => Ok(Self(
                match PkgPart::from_str(n) {
                    Ok(p) => p,
                    Err(e) => return Err(AnyError(e.to_string())),
                },
                match AnyVersion::from_str(v) {
                    Ok(w) => w,
                    Err(e) => return Err(AnyError(e.to_string())),
                },
            )),
            // take entire string as name and refer to latest version
            None => Ok(Self(
                match PkgPart::from_str(s) {
                    Ok(p) => p,
                    Err(e) => return Err(AnyError(e.to_string())),
                },
                AnyVersion::Latest,
            )),
        }
    }
}

impl std::fmt::Display for PartialIpSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.get_name(), SPEC_DELIM, self.get_version())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn compute_checksum() {
        let sum = Ip::compute_checksum(&PathBuf::from("./tests/env/project1/"));
        assert_eq!(
            sum,
            Sha256Hash::from_u32s([
                2472527351, 1678808787, 3321465315, 1927515725, 108238780, 2368649324, 2487325306,
                4053483655
            ])
        )
    }

    #[test]
    fn from_str_ip_spec() {
        let ip = format!("name{}1.0.0", SPEC_DELIM);

        assert_eq!(
            IpSpec::new(
                PkgPart::from_str("name").unwrap(),
                Version::from_str("1.0.0").unwrap()
            ),
            IpSpec::from_str(&ip).unwrap()
        );

        // // @note: errors due to invalid char for parsing PkgPart, but tests for
        // // extracting delimiter from RHS only once
        // let ip = format!("global{}local{}0.3.0", SPEC_DELIM, SPEC_DELIM);

        // assert_eq!(
        //     IpSpec::new(
        //         PkgPart::from_str(&format!("global{}local", SPEC_DELIM)).unwrap(),
        //         Version::from_str("0.3.0").unwrap()
        //     ),
        //     IpSpec::from_str(&ip).unwrap()
        // );
    }

    #[test]
    fn from_str_ip_spec_bad() {
        let ip = format!("name");

        assert_eq!(IpSpec::from_str(&ip).is_err(), true);
    }
}
