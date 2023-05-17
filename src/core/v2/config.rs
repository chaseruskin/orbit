use std::collections::HashMap;
use crate::core::v2::plugin::Plugins;
use crate::core::v2::protocol::Protocols;
use std::str::FromStr;
use crate::core::v2::manifest::FromFile;
use std::path::PathBuf;
use crate::util::anyerror::AnyError;
use std::error::Error;

use serde_derive::{Serialize, Deserialize};

pub const CONFIG_FILE: &str = "config.toml";

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Config {
    include: Option<Vec<String>>,
    env: Option<HashMap<String, String>>,
    plugin: Option<Plugins>,
    protocol: Option<Protocols>,
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl FromFile for Config {

    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(r) => Ok(r),
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!("failed to parse {} file: {}", path.display(), e)))?
            }
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            include: None,
            env: None,
            plugin: None,
            protocol: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const C_0: &str = r#"
# This is a blank configuration file.
"#;

    const C_1: &str = r#"
# orbit configuration file

# list of configuration files
include = [
    "path/to/other/config.toml",
]
    
[[plugin]]
alias = "quartus"
summary = "Complete toolflow for Intel Quartus Prime backend program"
command = "python"
args = ["./plugin/quartus.py"]
fileset.pin-plan = "*.board"
fileset.bdf-file = "*.bdf"

[env]
COURSE = "Reconfigurable Computing [EEL5721]"
QUARTUS_PATH = "C:/intelFPGA_lite/19.1/quartus/bin64/"
VCD_VIEWER = "dwfv"

[[protocol]]
name = "kstp"
command = "python"
args = ["./download.py"]
"#;

    #[test]
    fn parse_empty_config() {
        match Config::from_str(C_0) {
            Ok(r) => assert_eq!(r, Config::new()),
            Err(e) => { println!("{}", e); panic!("failed to parse") }
        }
    }

    #[test]
    fn parse_basic_config() {
        match Config::from_str(C_1) {
            Ok(r) => assert_ne!(r, Config::new()),
            Err(e) => { println!("{}", e); panic!("failed to parse") }
        }
    }
}