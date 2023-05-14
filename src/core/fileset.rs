use std::str::FromStr;
use glob::{Pattern, PatternError};
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

pub struct Filesets(Vec<Fileset>);

impl From<HashMap<String, Style>> for Filesets {
    fn from(value: HashMap<String, Style>) -> Self {
        Self(value.into_iter().map(|(n, p)| {
            Fileset {
                name: n,
                pattern: p,
            }
        }).collect())
    }
}

impl Filesets {
    // pub fn inner(&self) -> &Vec<Fileset> {
        
    // }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Fileset {
    name: String,
    pattern: Style,
}

#[derive(Debug, PartialEq)]
pub struct Style(Pattern);

impl Style {
    fn inner(&self) -> &Pattern {
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
        Ok(Style(Pattern::new(&("**/".to_owned() + s))?.into()))
    }
}

use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::de::{self};
use std::fmt;

impl<'de> Deserialize<'de> for Style {
    fn deserialize<D>(deserializer: D) -> Result<Style, D::Error>
        where D: de::Deserializer<'de>
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Style;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a glob-style pattern")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                
                match Style::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e))
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
            return Err(Self::Err::EmptyName)
        }
        Ok(Fileset {
            pattern: match Pattern::new(pattern) {
                // pattern must not be empty
                Ok(p) => if p.as_str().is_empty() {
                    return Err(Self::Err::EmptyPattern)
                } else {
                    p.into()
                },
                Err(e) => return Err(Self::Err::PatternError(pattern.to_string(), e))
            },
            name: Self::standardize_name(name),
        })
    }
}

impl Fileset {
    /// Create a new `Fileset` structure.
    pub fn new() -> Self {
        Fileset {
            name: String::new(),
            pattern: Pattern::new("*").unwrap().into(),
        }
    }

    /// Set the `Fileset` name.
    pub fn name(mut self, s: &str) -> Self {
        self.name = Self::standardize_name(s);
        self
    }

    /// Set the `Fileset` glob-style pattern.
    pub fn pattern(mut self, p: &str) -> Result<Self, PatternError>{
        self.pattern = Pattern::new(&("**/".to_owned() + p))?.into();
        Ok(self)
    }

    /// Standardizes the name to be UPPER-AND-HYPHENS.
    /// 
    /// The returned string is its own data (cloned from `s`).
    fn standardize_name(s: &str) -> String {
        s.to_uppercase().replace('_', "-")
    }

    /// Uses the given pattern to return a set of build files.
    pub fn collect_files<'a>(&self, files: &'a [String]) -> Vec<&'a String> {
        let match_opts = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        files.iter().filter_map(|f| {
            if self.pattern.inner().matches_with(&f, match_opts) == true {
                Some(f)
            } else {
                None
            }
        }).collect()
    }

    /// Access name.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Access pattern.
    pub fn get_pattern(&self) -> &Pattern {
        &self.pattern.inner()
    }

    /// Creates format for blueprint.tsv file.
    /// 
    /// The format goes FILESET_NAME\tFILE_NAME\tFILE_PATH
    pub fn to_blueprint_string(&self, file: &str) -> String {
        let filename = {
            // removes root path
            let filenode = if let Some((_, node)) = file.rsplit_once('/') {
                node
            } else {
                file
            };
            // removes file extenstion
            match filenode.rsplit_once('.') {
                Some((name, _)) => name,
                None => filenode
            }
        };
        format!("{}\t{}\t{}\n", self.name, filename, file)
    }
}

/// Checks if the `file` is a VHDL file (ending with .vhd or .vhdl).
pub fn is_vhdl(file: &str) -> bool {
    if let Some((_, ending)) = file.rsplit_once('.') {
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhd") ||
        crate::util::strcmp::cmp_ascii_ignore_case(ending, "vhdl")
    } else {
        false
    }
}

/// Checks against file patterns if the file is an rtl file.
pub fn is_rtl(file: &str) -> bool {
    let match_opts = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let p1 = Pattern::new("*.vhd").unwrap();
    let p2 = Pattern::new("*.vhdl").unwrap();

    let tb1 = Pattern::new("tb_*").unwrap();
    let tb2 = Pattern::new("*_tb.*").unwrap();

    (p1.matches_with(file, match_opts) == true || p2.matches_with(file, match_opts) == true) && 
        tb1.matches_with(file, match_opts) == false && tb2.matches_with(file, match_opts) == false
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_blueprint_string() {
        let fset = Fileset::new().name("vhdl").pattern("*.sv").unwrap();
        let filepath = "c:/users/chase/develop/project/adder.sv";
        assert_eq!(fset.to_blueprint_string(&filepath), format!("VHDL\tadder\t{}\n", filepath));

        let filepath = "FILE2.sv";
        assert_eq!(fset.to_blueprint_string(&filepath), format!("VHDL\tFILE2\t{}\n", filepath));
    }

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
    fn fset_from_str() {
        let s = "xsim-cfg=*.wcfg";
        let fset = Fileset::from_str(s);
        assert_eq!(fset.unwrap(), Fileset {
            name: "XSIM-CFG".to_string(),
            pattern: Pattern::new("*.wcfg").unwrap().into()
        });

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
    }
}