//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use crate::error::Error;
use serde::de;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use crate::util::anyerror::Fault;

#[derive(Debug, PartialEq, Clone)]
pub struct Uuid {
    inner: uuid::Uuid,
    raw: Option<String>,
}

impl Uuid {
    pub fn new() -> Self {
        let mut id = Self {
            inner: uuid::Uuid::new_v4(),
            raw: None,
        };
        id.raw = Some(id.encode());
        id
    }

    pub fn nil() -> Self {
        Self {
            inner: uuid::Uuid::nil(),
            raw: Some("1".repeat(22)),
        }
    }

    pub fn get(&self) -> &uuid::Uuid {
        &self.inner
    }

    pub fn to_string_short(&self) -> String {
        format!("{:x?}", self.inner.to_fields_le().0.to_be())
    }

    /// Encodes the UUID into a base58 string.
    pub fn encode(&self) -> String {
        if let Some(r) = &self.raw {
            r.clone()
        } else {
            let bytes = self.inner.as_bytes();
            if bytes == &[0; 16] {
                "1".repeat(22)
            } else {
                let result = bs58::encode(bytes).into_string();
                match result.len() < 22 {
                    true => format!("{}{}", "1".repeat(22 - result.len()), result),
                    false => result,
                }
            }
        }
    }

    /// Decodes the UUID from a base58 string.
    pub fn decode(s: &str) -> Result<Self, Fault> {
        // convert the string into bytes
        if s.len() != 22 {
            return Err(Error::IdNot22Chars(s.len()))?;
        }
        let all_bytes = bs58::decode(s).into_vec()?;
        // println!("all bytes: {}", all_bytes.len());
        let mut bytes: [u8; 16] = [0; 16];
        if all_bytes.len() > 16 {
            for i in 0..16 {
                bytes[i] = all_bytes[i + all_bytes.len() - 16];
            }
        } else {
            for i in 0..16 {
                bytes[i] = all_bytes[i];
            }
        }

        Ok(Self {
            inner: uuid::Uuid::from_bytes(bytes),
            raw: Some(s.to_string()),
        })
    }
}

impl FromStr for Uuid {
    type Err = Fault;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw.as_ref().unwrap())
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
                match Uuid::decode(v) {
                    Ok(r) => Ok(r),
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
        serializer.serialize_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_encode_uuid() {
        let id = Uuid::new();
        println!("{}", id.inner.as_simple().to_string());
        let og_id = id.inner.as_simple().to_string();
        println!("{}", id.encode());
        println!("{}", id.encode().len());
        let s = id.encode();
        let id = Uuid::decode(&s).unwrap();
        println!("{}", id.inner.as_simple().to_string());
        let new_id = id.inner.as_simple().to_string();
        assert_eq!(og_id, new_id);
        // panic!();
    }

    #[test]
    fn ut_encode_nil() {
        let id = Uuid::nil();

        println!("{}", id.inner.as_simple().to_string());
        let og_id = id.inner.as_simple().to_string();
        println!("{}", id.encode());
        println!("{}", id.encode().len());
        let s = id.encode();
        let id = Uuid::decode(&s).unwrap();
        println!("{}", id.inner.as_simple().to_string());
        let new_id = id.inner.as_simple().to_string();
        assert_eq!(og_id, new_id);
        // panic!();
    }

    #[test]
    fn ut_user_given_id() {
        let user_value = "THiSisMYuniQueidseeit8";
        let id = Uuid::from_str(user_value).unwrap();
        assert_eq!(user_value, id.encode());
    }
}
