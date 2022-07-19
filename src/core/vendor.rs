use crate::{core::manifest::Manifest, util::{anyerror::{Fault, AnyError}, filesystem::{normalize_path, self}}};
use std::{path::PathBuf, collections::HashMap, str::FromStr};
use crate::core::pkgid::PkgId;
use super::{pkgid::PkgPart, config::FromToml, manifest::IpManifest, version::Version, hook::Hook, variable::{VariableTable}, template};
use std::io::Write;

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
    hooks: HookTable,
}

impl VendorToml {
    fn new() -> Self {
        Self { vendor: Vendor::new(), hooks: HookTable::new() }
    }
}

impl FromToml for VendorToml {
    type Err = Fault;

    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            vendor: Vendor::from_toml(table.get("vendor").unwrap().as_table().unwrap())?,
            hooks: if let Some(tbl) = table.get("hook") { HookTable::from_toml(tbl.as_table().unwrap())? } else { HookTable::new() },
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct HookTable {
    pre_publish: Option<String>,
    post_publish: Option<String>,
}

impl HookTable {
    pub fn new() -> Self {
        Self { pre_publish: None, post_publish: None }
    }
}

impl FromToml for HookTable {
    type Err = Fault;
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized {
        Ok(Self {
            pre_publish: Self::get(&table, "pre-publish")?,
            post_publish: Self::get(&table, "post-publish")?,
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

    /// References the vendor's name
    pub fn get_name(&self) -> &PkgPart {
        &self.vendor.vendor.name
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

    /// Copies the ip manifest into the vendor.
    pub fn publish(&self, ip: &mut IpManifest, next: &Version, vtable: VariableTable) -> Result<(), Fault> {
        let cwd = std::env::current_dir()?;
        std::env::set_current_dir(self.get_root())?;

        self.pre_publish_hook(&vtable)?;

        // create the path to write to destination
        let pkgid = ip.get_pkgid();

        // create intermediate directories
        let pub_dir = self.get_root()
            .join(pkgid.get_library().as_ref().unwrap())
            .join(pkgid.get_name());
        std::fs::create_dir_all(&pub_dir)?;

        // serialize unit data
        ip.stash_units();

        // write contents to new file location
        {
            let mut pub_file = std::fs::File::create(&pub_dir.join(format!("Orbit-{}.toml", next)))?;
            pub_file.write(ip.get_manifest().get_doc().to_string().as_bytes())?;
        }
        
        self.post_publish_hook(&vtable)?;

        std::env::set_current_dir(cwd)?;
        Ok(())
    }

    fn get_hook(&self, hook: &Option<String>, vtable: &VariableTable) -> Result<Option<Hook>, Fault> {
        // check if the key is in manifest
        let file = if let Some(h) = hook.as_ref() { h } else { return Ok(None) };
        // check if file exists
        let r_path = PathBuf::from(filesystem::resolve_rel_path(&self.get_root(), file.to_string()));
        let contents = std::fs::read_to_string(r_path)?;
        // perform variable replacement
        Ok(Some(Hook::from_str(&template::substitute(contents, vtable)).unwrap()))
    }

    fn pre_publish_hook(&self, vtable: &VariableTable) -> Result<(), Fault>  {
        let hook = self.get_hook(&self.vendor.hooks.pre_publish, vtable)?;
        if let Some(h) = hook {
            h.execute()?;
        }
        Ok(())
    }

    fn post_publish_hook(&self, vtable: &VariableTable) -> Result<(), Fault>  {
        let hook = self.get_hook(&self.vendor.hooks.post_publish, vtable)?;
        if let Some(h) = hook {
            h.execute()?;
        }
        Ok(())
    }

    /// Pulls and pushes the underlying git repository, if it exists.
    pub fn sync(&self) -> Result<(), Fault> {
        todo!()
    }
}