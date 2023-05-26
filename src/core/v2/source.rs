use std::str::FromStr;
use crate::util::anyerror::AnyError;
use crate::core::v2::protocol::Name;

type ProtocolName = Name;

/// A [Source] outlines the process and location for extracting packages from the internet.
#[derive(Debug, PartialEq, Clone)]
pub struct Source {
    protocol: Option<ProtocolName>,
    url: String,
}

impl Source {
    pub fn new() -> Self {
        Self {
            protocol: None,
            url: String::new(),
        }
    }

    pub fn get_protocol(&self) -> &Option<ProtocolName> {
        &self.protocol
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn is_default(&self) -> bool {
        self.protocol.is_none()
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.protocol {
            Some(p) => {
                write!(f, "{}{}{}", p.as_ref(), DELIM, self.url)
            },
            None => {
                write!(f, "{}", self.url)
            },
        }
        
    }
}

pub const DELIM: &str = "+";

impl FromStr for Source {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // check if the delimiter exists
        match s.split_once(DELIM) {
            Some((proto, url)) => { 
                Ok(Self {
                    protocol: Some(ProtocolName::from_str(proto)?),
                    url: String::from(url),
                })
            },
            None => {
                Ok(Self {
                    protocol: None,
                    url: String::from(s),
                })
            },
        }
    }
}

use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::de::{self};
use std::fmt;

impl<'de> Deserialize<'de> for Source {
    fn deserialize<D>(deserializer: D) -> Result<Source, D::Error>
        where D: de::Deserializer<'de>
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Source;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a semantic version number")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                
                match Source::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let src: &str = "cfs+https://some.url";

        assert_eq!(Source::from_str(src).unwrap(), Source {
            protocol: Some(ProtocolName::from_str("cfs").unwrap()),
            url: String::from("https://some.url"),
        });

        let src: &str = "https://some.url";

        assert_eq!(Source::from_str(src).unwrap(), Source {
            protocol: None,
            url: String::from("https://some.url"),
        });
    }
}