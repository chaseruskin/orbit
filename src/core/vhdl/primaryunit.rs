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

#[derive(Debug)]
pub struct Unit {
    name: Identifier,
    symbol: VHDLSymbol
}

impl std::hash::Hash for Unit {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Unit {}

use std::collections::HashMap;

pub fn collect_units(files: &Vec<String>) -> HashMap<PrimaryUnit, String> {
    let mut result = HashMap::new();
    for source_file in files {
        if crate::core::fileset::is_vhdl(&source_file) == true {
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = VHDLParser::read(&contents).into_symbols();
            // transform into primary design units
            symbols.into_iter().filter_map(|sym| {
                let name = sym.as_iden()?.clone();
                match sym {
                    VHDLSymbol::Entity(_) => Some(PrimaryUnit::Entity(Unit{ name: name, symbol: sym })),
                    VHDLSymbol::Package(_) => Some(PrimaryUnit::Package(Unit{ name: name, symbol: sym })),
                    VHDLSymbol::Configuration(_) => Some(PrimaryUnit::Configuration(Unit{ name: name, symbol: sym })),
                    VHDLSymbol::Context(_) => Some(PrimaryUnit::Context(Unit{ name: name, symbol: sym })),
                    _ => None,
                }
            }).for_each(|e| {
                result.insert(e, source_file.clone());
            });
        }
    }
    result
}