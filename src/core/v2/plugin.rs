//! A plugin is a user-defined backend workflow for processing the files collected
//! in the generated blueprint file.

use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::core::fileset::Style;

type Filesets = HashMap<String, Style>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    alias: String,
    command: String,
    args: Option<Vec<String>>,
    fileset: Option<Filesets>,
    summary: Option<String>,
    details: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl Plugin {
    pub fn get_filesets(&self) -> Option<&Filesets> {
        self.fileset.as_ref()
    }
}

impl FromStr for Plugin {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Plugins {
    protocol: Vec<Plugin>
}

impl FromStr for Plugins {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const P_1: &str = r#" 
alias = "ghdl"
summary = "Backend script for simulating VHDL with GHDL."  
command = "python"
args = ["./scripts/ghdl.py"]
fileset.py-model = "{{orbit.bench}}.py"
fileset.text = "*.txt"
"#;

    const P_2: &str = r#"
alias = "ffi"
command = "bash"
args = ["~/scripts/download.bash"]    
"#;

    #[test]
    fn from_toml_string() {
        let plug = Plugin::from_str(P_1).unwrap();
        assert_eq!(plug, Plugin {
            alias: String::from("ghdl"),
            command: String::from("python"),
            args: Some(vec![String::from("./scripts/ghdl.py")]),
            summary: Some(String::from("Backend script for simulating VHDL with GHDL.")),
            fileset: Some(HashMap::from([
                (String::from("py-model"), Style::from_str("{{orbit.bench}}.py").unwrap()),
                (String::from("text"), Style::from_str("*.txt").unwrap()),
            ])),
            details: None,
            root: None,
        });

        let plug = Plugin::from_str(P_2).unwrap();
        assert_eq!(plug, Plugin {
            alias: String::from("ffi"),
            command: String::from("bash"),
            args: Some(vec![String::from("~/scripts/download.bash")]),
            summary: None,
            fileset: None,
            details: None,
            root: None,
        });
    }

    // #[test]
    // fn series_of_protocols() {
    //     let contents = format!("{0}{1}\n{0}{2}", "[[protocol]]", P_1, P_2);
    //     // assemble the list of protocols
    //     let protos = Protocols::from_str(&contents).unwrap();
    //     assert_eq!(protos, Protocols {
    //         protocol: vec![
    //             Protocol::from_str(P_1).unwrap(),
    //             Protocol::from_str(P_2).unwrap()
    //         ],
    //     });
    // }
}