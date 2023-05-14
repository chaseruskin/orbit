//! A protocol is a series of steps defined for requesting files/packages
//! from the internet.

use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Protocol {
    name: String,
    command: String,
    args: Option<Vec<String>>,
}

impl FromStr for Protocol {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

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

#[cfg(test)]
mod test {
    use super::*;

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
        });

        let proto = Protocol::from_str(P_2).unwrap();
        assert_eq!(proto, Protocol {
            name: String::from("ffi"),
            command: String::from("bash"),
            args: Some(vec![String::from("~/scripts/download.bash")]),
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