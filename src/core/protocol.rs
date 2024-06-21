//! A protocol is a series of steps defined for requesting files/packages
//! from the internet.

use crate::core::target::Process;
use crate::core::variable;
use crate::util::filesystem::Standardize;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

pub type Protocols = Vec<Protocol>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    name: String,
    command: String,
    args: Option<Vec<String>>,
    description: Option<String>,
    explanation: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl FromStr for Protocol {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl Protocol {
    /// Performs variable substitution on the provided arguments for the protocol.
    pub fn replace_vars_in_args(mut self, vtable: &VariableTable) -> Self {
        self.args = if let Some(args) = self.args {
            Some(
                args.into_iter()
                    .map(|arg| variable::substitute(arg, vtable))
                    .collect(),
            )
        } else {
            self.args
        };
        self
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
            None => Vec::new(),
        }
    }
}

use crate::commands::orbit::UpgradeError;
use crate::commands::orbit::RESPONSE_OKAY;
use crate::util::anyerror::Fault;
use curl::easy::Easy;
use std::io::Write;
use tempfile;
use zip::ZipArchive;

use super::variable::VariableTable;

impl Protocol {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            root: None,
            args: None,
            description: None,
            explanation: None,
        }
    }

    pub fn set_root(&mut self, root: PathBuf) {
        self.root = Some(root);
    }

    pub fn get_name(&self) -> &str {
        &self.name.as_ref()
    }

    /// Creates a string to display a list of plugins.
    ///
    /// The string lists the plugins in alphabetical order by `name`.
    pub fn list_protocols(protos: &mut [&&Protocol]) -> String {
        let mut list = String::from("Protocols:\n");
        protos.sort_by(|a: &&&Protocol, b| a.name.cmp(&b.name));
        for proto in protos {
            list += &format!("  {}\n", proto.quick_info());
        }
        list
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self) -> String {
        format!(
            "{:<16}{}",
            self.name,
            self.description.as_ref().unwrap_or(&String::new())
        )
    }

    /// Performs the default behavior for a protocol.
    ///
    /// This will attempt to download the url as a zip file and extract it to
    /// its queue directory.
    pub fn single_download(url: &str, dst: &PathBuf) -> Result<(), Fault> {
        let mut body_bytes = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        body_bytes.extend_from_slice(data);
                        Ok(data.len())
                    })
                    .unwrap();

                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(
                    url.to_string(),
                    rc,
                )));
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

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
name:    {}
command: {} {}
root:    {}
{}{}",
            self.name,
            self.command,
            self.args
                .as_ref()
                .unwrap_or(&Vec::new())
                .iter()
                .fold(String::new(), |x, y| { x + "\"" + &y + "\" " }),
            PathBuf::standardize(self.root.as_ref().unwrap()).display(),
            {
                if let Some(text) = &self.description {
                    format!("\n{}\n", text)
                } else {
                    String::new()
                }
            },
            {
                if let Some(text) = &self.explanation {
                    format!("\n{}", text)
                } else {
                    String::new()
                }
            },
        )
    }
}

use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum ProtocolError {
    Missing(String),
}

impl Error for ProtocolError {}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(name) => write!(
                f,
                "No protocol named '{}'\n\nTry `orbit install --list` to see available protocols",
                name
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    pub struct Protocols {
        protocol: Vec<Protocol>,
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
        assert_eq!(
            proto,
            Protocol {
                name: String::from("gcp"),
                command: String::from("python"),
                args: None,
                root: None,
                description: None,
                explanation: None,
            }
        );

        let proto = Protocol::from_str(P_2).unwrap();
        assert_eq!(
            proto,
            Protocol {
                name: String::from("ffi"),
                command: String::from("bash"),
                args: Some(vec![String::from("~/scripts/download.bash")]),
                root: None,
                description: None,
                explanation: None,
            }
        );
    }

    #[test]
    fn series_of_protocols() {
        let contents = format!("{0}{1}\n{0}{2}", "[[protocol]]", P_1, P_2);
        // assemble the list of protocols
        let protos = Protocols::from_str(&contents).unwrap();
        assert_eq!(
            protos,
            Protocols {
                protocol: vec![
                    Protocol::from_str(P_1).unwrap(),
                    Protocol::from_str(P_2).unwrap()
                ],
            }
        );
    }
}
