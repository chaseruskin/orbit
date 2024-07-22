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

use serde::de;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
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
            inner: uuid::Uuid::nil(),
        }
    }

    pub fn get(&self) -> &uuid::Uuid {
        &self.inner
    }

    pub fn to_string_short(&self) -> String {
        format!("{:x?}", self.inner.to_fields_le().0.to_be())
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
