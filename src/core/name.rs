//
//  Copyright (C) 2022-2025  Chase Ruskin
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

use serde::{de, Deserialize};
use serde_derive::Serialize;
use std::error::Error;
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, PartialOrd, Clone, Eq, Hash, Serialize, Ord)]
#[serde(transparent)]
pub struct Name(String);

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Name, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a project name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Name::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Name {
    pub fn new() -> Self {
        Name(String::new())
    }

    /// Normalizes the identifier by converting `-` to `_` and all lowercase letters.
    pub fn to_normal(&self) -> Name {
        Name(self.0.replace('-', "_").to_lowercase())
    }

    pub fn as_ref(&self) -> &str {
        &self.0
    }

    /// Checks if the current [PkgPart] is a superset of the `rhs`.
    pub fn contains(&self, rhs: &Self) -> bool {
        self.to_normal()
            .to_string()
            .contains(rhs.to_normal().as_ref())
    }

    /// Checks if the current [PkgPart] is a superset of the `rhs` starting
    /// from position 0.
    pub fn starts_with(&self, rhs: &Self) -> bool {
        self.to_normal()
            .to_string()
            .starts_with(rhs.to_normal().as_ref())
    }
}

impl AsRef<std::path::Path> for Name {
    fn as_ref(&self) -> &std::path::Path {
        self.0.as_ref()
    }
}

impl From<&Name> for toml_edit::Value {
    fn from(p: &Name) -> Self {
        From::<&String>::from(&p.0)
    }
}

impl std::str::FromStr for Name {
    type Err = NameError;

    /// Verifies a part follows the `PkgId` specification.
    ///
    /// First character must be `alphabetic`. Remaining characters must be
    /// `ascii alphanumeric`, `-`, or `_`.
    fn from_str(s: &str) -> Result<Self, NameError> {
        use NameError::*;

        // Check to make sure the name is not "work"
        if s.to_ascii_lowercase() == "work" {
            return Err(NameError::ReservedName(s.to_string()));
        }

        if let Some(c) = s.chars().next() {
            if c.is_ascii_alphabetic() == false {
                return Err(NotAlphabeticFirst(c));
            }
        }
        // find first char in pkgid part not following spec
        let result = s
            .chars()
            .find(|&c| !c.is_ascii_alphanumeric() && !(c == '_') && !(c == '-'));
        if let Some(r) = result {
            Err(InvalidChar(r))
        } else {
            // verify the last char
            if let Some(c) = s.chars().last() {
                if c == '_' || c == '-' {
                    return Err(InvalidEnding);
                }
            }
            Ok(Self(s.to_owned()))
        }
    }
}

impl std::cmp::PartialEq for Name {
    /// Two `PkgId`'s are considered equivalent if they have identical case
    /// insensitive string parts. Different than `==` operator. Converting '-'
    /// to '_' is also applied.
    fn eq(&self, other: &Self) -> bool {
        self.to_normal().0 == other.to_normal().0
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum NameError {
    NotAlphabeticFirst(char),
    BadLen(String, usize),
    Empty,
    InvalidChar(char),
    MissingVendor,
    MissingLibrary,
    InvalidEnding,
    ReservedName(String),
}

impl Error for NameError {}

impl Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use NameError::*;
        match self {
            ReservedName(w) => write!(f, "identifier \"{}\" is a reserved name", w),
            NotAlphabeticFirst(ch) => write!(
                f,
                "expects first character to be alphabetic but found '{}'",
                ch
            ),
            InvalidChar(ch) => write!(
                f,
                "character '{}' is not alphanumeric, a dash, or an underscore",
                ch
            ),
            Empty => write!(f, "cannot be empty"),
            BadLen(id, len) => write!(
                f,
                "bad length for pkgid \"{}\"; expecting 3 parts but found {}",
                id, len
            ),
            MissingLibrary => write!(f, "missing library part"),
            MissingVendor => write!(f, "missing vendor part"),
            InvalidEnding => write!(f, "expects last character to not be a dash or underscore"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate() {
        //okays
        let s = "name";
        assert_eq!(Name::from_str(s), Ok(Name(s.to_owned())));
        let s = "NAME_1";
        assert_eq!(Name::from_str(s), Ok(Name(s.to_owned())));
        let s = "NAME_1-0";
        assert_eq!(Name::from_str(s), Ok(Name(s.to_owned())));
        let s = "N9A-ME_1N--A432ME";
        assert_eq!(Name::from_str(s), Ok(Name(s.to_owned())));

        //errors
        assert!(Name::from_str("ven dor").is_err());
        assert!(Name::from_str("2name").is_err());
        assert!(Name::from_str("_name").is_err());
        assert!(Name::from_str("-name").is_err());
        assert!(Name::from_str("path/name").is_err());
        assert!(Name::from_str("na!me").is_err());
    }
}
