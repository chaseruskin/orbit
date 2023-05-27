//! A protocol is a series of steps defined for requesting files/packages
//! from the internet.

use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use crate::core::v2::plugin::Process;
use std::path::PathBuf;

pub type Protocols = Vec<Protocol>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    name: String,
    command: String,
    args: Option<Vec<String>>,
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl FromStr for Protocol {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Process for Protocol {
    fn get_root(&self) -> &PathBuf { 
        &self.root.as_ref().unwrap()
    }

    fn get_command(&self) -> &String {
        &self.command
    }

    fn get_args(&self) -> Vec<&String> {
        match &self.args {
            Some(list) => list.iter().map(|e| e).collect(),
            None => Vec::new()
        }
    }
}

use crate::commands::orbit::RESPONSE_OKAY;
use crate::commands::orbit::UpgradeError;
use crate::util::anyerror::Fault;
use curl::easy::Easy;
use zip::ZipArchive;
use tempfile;
use std::io::Write;

impl Protocol {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            root: None,
            args: None,
        }
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }

    pub fn get_name(&self) -> &str {
        &self.name.as_ref()
    }

    // /// Performs the default behavior of the download on each
    // pub fn default_execute(srcs: &[&String]) -> Result<(), Fault> {
        
    // }

    /// Performs the default behavior for a protocol.
    pub fn single_download(url: &str, dst: &PathBuf) -> Result<(), Fault> {
        let mut body_bytes = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer.write_function(|data| {
                    body_bytes.extend_from_slice(data);
                    Ok(data.len())
                }).unwrap();
        
                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(url.to_string(), rc)));
            }
        }
        // place the bytes into a file
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&body_bytes)?;
        let mut zip_archive = ZipArchive::new(temp_file)?;

        // decompress the zip file to the queue
        zip_archive.extract(&dst)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Protocols {
        protocol: Vec<Protocol>
    }
    
    impl FromStr for Protocols {
        type Err = toml::de::Error;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            toml::from_str(s)
        }
    }

    const P_1: &str = r#" 
name = "gcp"
command = "python"    
"#;

    const P_2: &str = r#"
name = "ffi"
command = "bash"
args = ["~/scripts/download.bash"]    
"#;

    #[test]
    fn from_toml_string() {
        let proto = Protocol::from_str(P_1).unwrap();
        assert_eq!(proto, Protocol {
            name: String::from("gcp"),
            command: String::from("python"),
            args: None,
            root: None,
        });

        let proto = Protocol::from_str(P_2).unwrap();
        assert_eq!(proto, Protocol {
            name: String::from("ffi"),
            command: String::from("bash"),
            args: Some(vec![String::from("~/scripts/download.bash")]),
            root: None,
        });
    }

    #[test]
    fn series_of_protocols() {
        let contents = format!("{0}{1}\n{0}{2}", "[[protocol]]", P_1, P_2);
        // assemble the list of protocols
        let protos = Protocols::from_str(&contents).unwrap();
        assert_eq!(protos, Protocols {
            protocol: vec![
                Protocol::from_str(P_1).unwrap(),
                Protocol::from_str(P_2).unwrap()
            ],
        });
    }
}