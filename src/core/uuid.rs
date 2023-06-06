use serde::{Serialize, Deserialize};
use serde::Serializer;
use std::fmt;
use serde::de;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Uuid {
    inner: uuid::Uuid,
}

impl Uuid {
    pub fn new() -> Self {
        Self {
            inner: uuid::Uuid::new_v4(),
        } 
    }

    pub fn nil() -> Self {
        Self {
            inner: uuid::Uuid::nil()
        }
    }

    pub fn get(&self) -> &uuid::Uuid {
        &self.inner
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Uuid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a universal unique identifier")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {   
                match uuid::Uuid::from_str(v) {
                    Ok(v) => Ok(Uuid { inner: v }),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.inner.to_string())
    }
}
