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

use glob::{Pattern, PatternError};
use serde::de::MapAccess;
use serde_derive::Serialize;
use std::str::FromStr;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct Fileset {
    #[serde(skip_serializing, skip_deserializing)]
    name: String,
    patterns: Vec<Style>,
}

use serde::de::Visitor;

impl<'de> serde::Deserialize<'de> for Fileset {
    fn deserialize<D>(deserializer: D) -> Result<Fileset, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        enum Field {
            Patterns,
        }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`patterns`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "patterns" => Ok(Field::Patterns),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        // This is a Visitor that forwards string types to T's `FromStr` impl and
        // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
        // keep the compiler from complaining about T being an unused generic type
        // parameter. We need T in order to know the Value type for the Visitor
        // impl.
        struct LayerVisitor;

        impl<'de> Visitor<'de> for LayerVisitor {
            type Value = Fileset;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Fileset, E>
            where
                E: de::Error,
            {
                Ok(Fileset {
                    name: String::new(),
                    patterns: vec![match Style::from_str(value) {
                        Ok(v) => v,
                        Err(e) => return Err(de::Error::custom(e))?,
                    }],
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Fileset, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut patterns: Option<Vec<Style>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Patterns => {
                            if patterns.is_some() {
                                return Err(de::Error::duplicate_field("patterns]"));
                            }
                            patterns = Some(map.next_value()?);
                        }
                    }
                }
                let patterns = patterns.unwrap();
                Ok(Fileset {
                    name: String::new(),
                    patterns: patterns,
                })
            }
        }

        const FIELDS: &[&str] = &["patterns"];
        deserializer.deserialize_struct("Fileset", FIELDS, LayerVisitor)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Style(Pattern);

impl Style {
    pub fn inner(&self) -> &Pattern {
        &self.0
    }
}

impl From<Pattern> for Style {
    fn from(value: Pattern) -> Self {
        Self(value)
    }
}

impl FromStr for Style {
    type Err = PatternError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefix = match s.get(0..1) {
            Some(".") => "",
            _ => "**/",
        };
        Ok(Style(Pattern::new(&(prefix.to_owned() + s))?.into()))
    }
}

use serde::de::{self};
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D>(deserializer: D) -> Result<Style, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Style;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a glob-style pattern")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Style::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Style {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Debug)]
pub enum FilesetError {
    MissingSeparator(char),
    EmptyPattern,
    EmptyName,
    PatternError(String, PatternError),
}

impl std::error::Error for FilesetError {}

impl std::fmt::Display for FilesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::EmptyPattern => write!(f, "empty pattern"),
            Self::EmptyName => write!(f, "empty name"),
            Self::MissingSeparator(c) => write!(f, "missing separator '{}'", c),
            Self::PatternError(p, e) => write!(f, "'{}' {}", p, e.to_string().to_lowercase()),
        }
    }
}

impl FromStr for Fileset {
    type Err = FilesetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split by '=' sign (or ':'?)
        let result = s.split_once('=');
        if result == None {
            return Err(Self::Err::MissingSeparator('='));
        }
        let (name, pattern) = result.unwrap();
        // name cannot be empty
        if name.is_empty() {
            return Err(Self::Err::EmptyName);
        }
        Ok(Fileset {
            patterns: match Pattern::new(pattern) {
                // pattern must not be empty
                Ok(p) => {
                    if p.as_str().is_empty() {
                        return Err(Self::Err::EmptyPattern);
                    } else {
                        vec![p.into()]
                    }
                }
                Err(e) => return Err(Self::Err::PatternError(pattern.to_string(), e)),
            },
            name: Self::standardize_name(name),
        })
    }
}

impl Fileset {
    /// Create a new [Fileset] structure.
    pub fn new() -> Self {
        Fileset {
            name: String::new(),
            patterns: Vec::new(),
        }
    }

    /// Set the `Fileset` name.
    pub fn name(mut self, s: &str) -> Self {
        self.name = Self::standardize_name(s);
        self
    }

    /// Set the [Fileset] glob-style pattern.
    ///
    /// If no explicit relative file path character is present (`.`), then
    /// it implicitly sets a recursive directory glob pattern as the prefix
    /// (`**/`).
    pub fn add_pattern(mut self, p: &str) -> Result<Self, PatternError> {
        let prefix = match p.get(0..1) {
            Some(".") => "",
            _ => "**/",
        };
        self.patterns
            .push(Pattern::new(&(prefix.to_owned() + p))?.into());
        Ok(self)
    }

    /// Standardizes the name to be COBOL-CASE.
    ///
    /// The returned string is its own data (cloned from `s`).
    pub fn standardize_name(s: &str) -> String {
        s.to_uppercase().replace('_', "-")
    }

    /// Uses the given pattern to return a set of build files.
    pub fn collect_files<'a>(&self, files: &'a [String]) -> Vec<&'a String> {
        let match_opts = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        files
            .iter()
            .filter_map(|f| {
                // iterate through all known patterns for the fileset
                match &self
                    .patterns
                    .iter()
                    .find(|p| p.inner().matches_with(&f, match_opts))
                {
                    Some(_) => Some(f),
                    None => None,
                }
            })
            .collect()
    }

    /// Access name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Access the underlying patterns.
    pub fn get_patterns(&self) -> Vec<&Pattern> {
        self.patterns.iter().map(|f| f.inner()).collect()
    }
}

/// Checks if the `file` is a VHDL file (ending with .vhd or .vhdl).
pub fn is_vhdl(file: &str) -> bool {
    if let Some((_, ending)) = file.rsplit_once('.') {
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhd")
            || crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhdl")
    } else {
        false
    }
}

/// Checks if the `file` is a Verilog file (ending with .v, .vlg, .vl).
pub fn is_verilog(file: &str) -> bool {
    if let Some((_, ending)) = file.rsplit_once('.') {
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "v")
            || crate::util::strcmp::cmp_ascii_ignore_case(ending, "vl")
            || crate::util::strcmp::cmp_ascii_ignore_case(ending, "vlg")
    } else {
        false
    }
}

/// Checks if the `file` is a Verilog file (ending with .sv).
pub fn is_systemverilog(file: &str) -> bool {
    if let Some((_, ending)) = file.rsplit_once('.') {
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "sv")
    } else {
        false
    }
}

/// Checks if the given file is one of the supported HDLs.
pub fn is_hdl(file: &str) -> bool {
    is_vhdl(file) || is_verilog(file) || is_systemverilog(file)
}

/// Checks against file patterns if the file is an rtl file (not testbench).
pub fn is_rtl(file: &str) -> bool {
    let match_opts = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    if is_hdl(file) == false {
        return false;
    }

    let tb1 = Pattern::new("tb_*").unwrap();
    let tb2 = Pattern::new("*_tb.*").unwrap();

    tb1.matches_with(file, match_opts) == false && tb2.matches_with(file, match_opts) == false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn detect_vhdl_files() {
        let s = "filename.vhd";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.VHD";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.VHdL";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.vhdl";
        assert_eq!(is_vhdl(s), true);

        let s = "filename.v";
        assert_eq!(is_vhdl(s), false);

        let s = "filename";
        assert_eq!(is_vhdl(s), false);

        let s = "filename.sv";
        assert_eq!(is_vhdl(s), false);
    }

    #[test]
    fn assemble_fileset() {
        let fset = Fileset::new()
            .name("hello_world")
            .add_pattern("*.txt")
            .unwrap();
        assert_eq!(
            fset,
            Fileset {
                name: String::from("HELLO-WORLD"),
                patterns: vec![Pattern::new("**/*.txt").unwrap().into()],
            }
        );

        let fset = Fileset::new()
            .name("hello_world")
            .add_pattern("./some/specific/path.txt")
            .unwrap();
        assert_eq!(
            fset,
            Fileset {
                name: String::from("HELLO-WORLD"),
                patterns: vec![Pattern::new("./some/specific/path.txt").unwrap().into()],
            }
        );
    }

    #[test]
    fn fset_from_str() {
        let s = "xsim-cfg=*.wcfg";
        let fset = Fileset::from_str(s);
        assert_eq!(
            fset.unwrap(),
            Fileset {
                name: String::from("XSIM-CFG"),
                patterns: vec![Pattern::new("*.wcfg").unwrap().into()],
            }
        );

        let s = "xsim-cfg=";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // empty pattern

        let s = "=*.txt";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // empty name

        let s = "xsim-cfg";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // missing separator

        let s = "xsim-cfg=[";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.is_err(), true); // pattern error
    }

    #[test]
    fn std_name() {
        let s: &str = "VHDL-RTL";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");

        let s: &str = "vhdl-rtl";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");

        let s: &str = "vhdl_rtl";
        assert_eq!(Fileset::standardize_name(s), "VHDL-RTL");

        let s: &str = "board_fiLe";
        assert_eq!(Fileset::standardize_name(s), "BOARD-FILE");
    }
}
