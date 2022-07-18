use crate::{core::manifest::Manifest, util::anyerror::Fault};
use std::{path::PathBuf, collections::HashMap};
use crate::core::pkgid::PkgId;
use super::{pkgid::PkgPart, config::FromToml};

#[derive(Debug, PartialEq)]
pub struct IndexTable(HashMap<PkgId, String>);

impl IndexTable {
    /// Updates the table with the registry's name.
    fn vendor(self, iden: &PkgPart) -> Self {
        let name = iden.as_ref();
        let mut new_map = HashMap::new();
        self.0.into_iter().map(|f| {
            (f.0.vendor(name).unwrap(), f.1)
        }).for_each(|entry| {
            new_map.insert(entry.0, entry.1);
        });
        Self(new_map)
    }
}

impl FromToml for IndexTable {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let mut pkgs = HashMap::new();
        // iterate and read through the index table
        for j in table {
            for k in table[j.0].as_table().unwrap() {
                // create pkgids
                pkgs.insert(PkgId::new()
                    .library(j.0).unwrap()
                    .name(k.0).unwrap(), 
                    k.1.as_str().unwrap().to_owned());
            }
        }
        Ok(Self(pkgs))
    }
}

#[derive(Debug, PartialEq)]
pub struct VendorToml {
    vendor: Vendor,
    index: IndexTable,
}

impl VendorToml {
    fn new() -> Self {
        Self { vendor: Vendor::new(), index: IndexTable(HashMap::new()) }
    }
}

impl FromToml for VendorToml {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let vendor = Vendor::from_toml(table.get("vendor").unwrap().as_table().unwrap())?;
        Ok(Self {
            index: IndexTable::from_toml(table.get("index").unwrap().as_table().unwrap())?.vendor(&vendor.name),
            vendor: vendor,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Vendor {
    name: PkgPart,
    summary: Option<String>,
    repository: Option<String>,
}

impl Vendor {
    pub fn new() -> Self {
        Self { name: PkgPart::new(), summary: None, repository: None }
    }
}

impl FromToml for Vendor {
    type Err = Fault;
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            name: Self::require(&table, "name")?,
            summary: Self::get(&table, "summary")?,
            repository: Self::get(&table, "repository")?,
        })
    }
}

#[derive(Debug)]
pub struct VendorManifest {
    vendor: VendorToml,
    manifest: Manifest,
}

pub const VENDOR_MANIFEST_FILE: &str = "index.toml";

impl VendorManifest {
    /// Reads off the entries of the index and converts them into `pkgids`.
    pub fn read_index(&self) -> Vec<PkgId> {
        let mut pkgs = Vec::new();
        // iterate and read through the index table
        let table = self.manifest.get_doc()["index"].as_table().unwrap();
        let vendor_name = &self.get_name();
        for j in table {
            for k in table[j.0].as_table().unwrap() {
                // create pkgids
                pkgs.push(PkgId::new()
                    .vendor(vendor_name).unwrap()
                    .library(j.0).unwrap()
                    .name(k.0).unwrap());
            }
        }
        pkgs
    }

    pub fn get_index(&self) -> &IndexTable {
        &self.vendor.index
    }

    /// Access the vendor's name
    pub fn get_name(&self) -> String {
        self.manifest.read_as_str("vendor", "name").unwrap()
    }

    /// Loads the manifest document from 
    pub fn from_path(file: &PathBuf) -> Result<Self, Fault> {
        // load the toml document
        let doc = Manifest::from_path(file.to_owned())?;
        Ok(Self {
            vendor: VendorToml::from_toml(&doc.get_doc().as_table())?,
            manifest: doc,
        })
    }
}


#[cfg(test)]
mod test {
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
        let tmp = tempfile::NamedTempFile::new().unwrap().path().to_path_buf();
        std::fs::write(&tmp, doc.as_bytes()).unwrap();
        let manifest = VendorManifest::from_path(&tmp).unwrap();

        let mut map = HashMap::new();
        map.insert(PkgId::from_str("ks-tech.rary.gates").unwrap(), "url1".to_owned());
        map.insert(PkgId::from_str("ks-tech.memory.ram").unwrap(), "url2".to_owned());

        assert_eq!(manifest.get_index().0, map);
    }
}