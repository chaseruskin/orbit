use crate::{core::manifest::Manifest, util::{anyerror::{Fault, AnyError}, filesystem::normalize_path}};
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
}

impl VendorToml {
    fn new() -> Self {
        Self { vendor: Vendor::new() }
    }
}

impl FromToml for VendorToml {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        let vendor = Vendor::from_toml(table.get("vendor").unwrap().as_table().unwrap())?;
        Ok(Self {
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
    pub fn get_manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn get_root(&self) -> PathBuf {
        self.manifest.get_path().parent().unwrap().to_path_buf()
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
            vendor:Self::wrap_toml(&doc, VendorToml::from_toml(&doc.get_doc().as_table()))?,
            manifest: doc,
        })
    }

    fn wrap_toml<T, E: std::fmt::Display>(m: &Manifest, r: Result<T, E>) -> Result<T, impl std::error::Error> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(AnyError(format!("vendor {}: {}", normalize_path(m.get_path().clone()).display(), e))),
        }
    }
}