use crate::util::anyerror::AnyError;
use serde_derive::Deserialize;
use std::str::FromStr;

/// A [Source] outlines the process and location for extracting packages from the internet.
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    protocol: Option<String>,
    url: String,
    #[serde(skip, default = "set_true")]
    valid: bool,
}

fn set_true() -> bool {
    true
}

impl Source {
    pub fn new() -> Self {
        Self {
            protocol: None,
            url: String::new(),
            valid: true,
        }
    }

    pub fn get_protocol(&self) -> &Option<String> {
        &self.protocol
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn is_default(&self) -> bool {
        self.protocol.is_none()
    }

    pub fn as_option(&self) -> Option<&Source> {
        match &self.valid {
            true => Some(&self),
            false => None,
        }
    }
}

impl From<Option<Source>> for Source {
    fn from(value: Option<Source>) -> Self {
        match value {
            Some(s) => s,
            None => Source::default(),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.protocol {
            Some(p) => {
                write!(f, "{}+{}", p, self.url)
            }
            None => {
                write!(f, "{}", self.url)
            }
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self {
            protocol: None,
            url: String::new(),
            valid: false,
        }
    }
}

impl FromStr for Source {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            url: s.to_string(),
            protocol: None,
            valid: true,
        })
    }
}

use serde::de::{self};
use serde::de::{MapAccess, Visitor};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

pub fn string_or_struct<'de, D>(deserializer: D) -> Result<Source, D::Error>
where
    D: de::Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct LayerVisitor;

    impl<'de> Visitor<'de> for LayerVisitor {
        type Value = Source;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Source, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<Source, M::Error>
        where
            M: MapAccess<'de>,
        {
            // falls back on the derived version of deser for the [Source] struct
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(LayerVisitor)
}

use serde::ser::SerializeMap;

impl Serialize for Source {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // check if needing to serialize
        match &self.valid {
            true => {
                // // serializer.serialize_str(&self.to_string()),
                let mut map = match self.get_protocol() {
                    Some(_) => serializer.serialize_map(Some(2)),
                    None => serializer.serialize_map(Some(1)),
                }?;

                map.serialize_entry("url", self.get_url())?;
                if let Some(p) = self.get_protocol() {
                    map.serialize_entry("protocol", p)?;
                }

                map.end()
            }
            false => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str() {
        let src: &str = "https://some.url";

        assert_eq!(
            Source::from_str(src).unwrap(),
            Source {
                protocol: None,
                url: String::from("https://some.url"),
                valid: true,
            }
        );
    }

    #[test]
    fn deser_struct() {
        let src: Source = match toml::from_str(EX1) {
            Ok(r) => r,
            Err(e) => panic!("{}", e.to_string()),
        };

        assert_eq!(src.is_valid(), true);
    }

    const EX1: &str = r#"url = "https://some.url"
protocol = "ktsp""#;
}
