use crate::core::manifest::Manifest;
use std::path::PathBuf;
use crate::core::pkgid::PkgId;

#[derive(Debug)]
pub struct VendorManifest(pub Manifest);

pub const VENDOR_MANIFEST_FILE: &str = "vendor.toml";

impl VendorManifest {

    /// Finds all Vendor manifest files along the provided path `path`.
    /// 
    /// Wraps Manifest::detect_all.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        Ok(Manifest::detect_all(path, VENDOR_MANIFEST_FILE)?.into_iter().map(|f| VendorManifest(f)).collect())
    }

    /// Reads off the entries of the index and converts them into `pkgids`.
    pub fn read_index(&self) -> Vec<PkgId> {
        let mut pkgs = Vec::new();
        // iterate and read through the index table
        let table = self.0.get_doc()["index"].as_table().unwrap();
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

    /// Access the vendor's name
    pub fn get_name(&self) -> String {
        self.0.read_as_str("vendor", "name").unwrap()
    }

}