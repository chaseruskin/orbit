use crate::core::vhdl::token::Identifier;

use super::symbol::VHDLSymbol;
use crate::core::vhdl::symbol::VHDLParser;

#[derive(Debug, PartialEq, Hash, Eq)]
pub enum PrimaryUnit {
    Entity(Unit),
    Package(Unit),
    Context(Unit),
    Configuration(Unit),
}

impl PrimaryUnit {
    /// Casts to an identifier. 
    /// 
    /// Currently is safe to unwrap in all instances.
    pub fn as_iden(&self) -> Option<&Identifier> {
        Some(match self {
            Self::Entity(u) => &u.name,
            Self::Package(u) => &u.name,
            Self::Context(u) => &u.name,
            Self::Configuration(u) => &u.name,
        })
    }
}

impl std::fmt::Display for PrimaryUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Entity(_) => "entity",
            Self::Package(_) => "package",
            Self::Context(_) => "context",
            Self::Configuration(_) => "configuration",
        })
    }
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub struct Unit {
    name: Identifier,
}

use std::collections::HashMap;

pub fn collect_units(files: &Vec<String>) -> HashMap<PrimaryUnit, String> {
    let mut result = HashMap::new();
    for source_file in files {
        if crate::core::fileset::is_vhdl(&source_file) == true {
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = VHDLParser::read(&contents).into_symbols();
            // transform into primary design units
            symbols.into_iter().filter_map(|f| {
                let name = f.as_iden()?.clone();
                match f {
                    VHDLSymbol::Entity(_) => Some(PrimaryUnit::Entity(Unit{ name: name })),
                    VHDLSymbol::Package(_) => Some(PrimaryUnit::Package(Unit{ name: name})),
                    VHDLSymbol::Configuration(_) => Some(PrimaryUnit::Configuration(Unit{ name: name })),
                    VHDLSymbol::Context(_) => Some(PrimaryUnit::Context(Unit{ name: name, })),
                    _ => None,
                }
            }).for_each(|e| {
                result.insert(e, source_file.clone());
            });
        }
    }
    result
}