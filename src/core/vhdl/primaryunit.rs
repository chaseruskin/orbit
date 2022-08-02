use toml_edit::InlineTable;
use crate::{core::{vhdl::token::Identifier, lexer::Position}, util::anyerror::Fault};
use std::{collections::HashMap, str::FromStr, path::PathBuf};
use crate::util::filesystem;
use super::symbol::VHDLSymbol;
use crate::core::vhdl::symbol::VHDLParser;

pub type PrimaryUnitStore = HashMap<Identifier, PrimaryUnit>;

#[derive(PartialEq, Hash, Eq)]
pub enum PrimaryUnit {
    Entity(Unit),
    Package(Unit),
    Context(Unit),
    Configuration(Unit),
}

impl PrimaryUnit {
    /// References the unit's identifier. 
    pub fn get_iden(&self) -> &Identifier {
        match self {
            Self::Entity(u) => &u.name,
            Self::Package(u) => &u.name,
            Self::Context(u) => &u.name,
            Self::Configuration(u) => &u.name,
        }
    }

    pub fn get_unit(&self) -> &Unit {
        match self {
            Self::Entity(unit) => unit,
            Self::Package(unit) => unit,
            Self::Context(unit) => unit,
            Self::Configuration(unit) => unit,
        }
    }

    /// Serializes the data into a toml inline table
    pub fn to_toml(&self) -> toml_edit::Value {
        let mut item = toml_edit::Value::InlineTable(InlineTable::new());
        let tbl = item.as_inline_table_mut().unwrap();
        tbl.insert("identifier", toml_edit::value(&self.get_iden().to_string()).into_value().unwrap());
        tbl.insert("type", toml_edit::value(&self.to_string()).into_value().unwrap());
        item
    }

    /// Deserializes the data from a toml inline table.
    pub fn from_toml(tbl: &toml_edit::InlineTable) -> Option<Self> {
        let unit = Unit {
            name: Identifier::from_str(tbl.get("identifier")?.as_str()?).unwrap(), 
            symbol: None,
            source: String::new(),
        };
        Some(match tbl.get("type")?.as_str()? {
            "entity" => Self::Entity(unit),
            "package" => Self::Package(unit),
            "context" => Self::Context(unit),
            "configuration" => Self::Configuration(unit),
            _ => return None,
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

pub struct Unit {
    name: Identifier,
    symbol: Option<VHDLSymbol>,
    /// source code file
    source: String,
}

impl Unit {
    pub fn get_symbol(&self) -> Option<&VHDLSymbol> {
        self.symbol.as_ref()
    }

    pub fn get_source_code_file(&self) -> &str {
        &self.source
    }
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

pub fn collect_units(files: &Vec<String>) -> Result<HashMap<Identifier, PrimaryUnit>, Fault> {
    let mut result: HashMap<Identifier, PrimaryUnit> = HashMap::new();
    // iterate through all source files
    for source_file in files {
        // only read the HDL files
        if crate::core::fileset::is_vhdl(&source_file) == true {
            // parse text into VHDL symbols
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = VHDLParser::read(&contents).into_symbols();
            // transform into primary design units
            let units: Vec<PrimaryUnit> = symbols.into_iter().filter_map(|sym| {
                let name = sym.as_iden()?.clone();
                match sym {
                    VHDLSymbol::Entity(_) => Some(PrimaryUnit::Entity(Unit{ name: name, symbol: Some(sym), source: source_file.clone() })),
                    VHDLSymbol::Package(_) => Some(PrimaryUnit::Package(Unit{ name: name, symbol: Some(sym), source: source_file.clone() })),
                    VHDLSymbol::Configuration(_) => Some(PrimaryUnit::Configuration(Unit{ name: name, symbol: Some(sym), source: source_file.clone() })),
                    VHDLSymbol::Context(_) => Some(PrimaryUnit::Context(Unit{ name: name, symbol: Some(sym), source: source_file.clone() })),
                    _ => None,
                }
            }).collect();
            
            for primary in units {
                if let Some(dupe) = result.insert(primary.get_iden().clone(), primary) {
                    return Err(DuplicateIdentifierError(
                        dupe.get_iden().clone(), 
                        PathBuf::from(source_file), 
                        result.get(dupe.get_iden()).unwrap().get_unit().get_symbol().unwrap().get_position().clone(),
                        PathBuf::from(dupe.get_unit().get_source_code_file()), 
                        dupe.get_unit().get_symbol().unwrap().get_position().clone(),
                    ))?;
                }
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]
pub struct DuplicateIdentifierError(pub Identifier, pub PathBuf, pub Position, pub PathBuf, pub Position);

impl std::error::Error for DuplicateIdentifierError {}

impl std::fmt::Display for DuplicateIdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_dir = std::env::current_dir().unwrap();
        let location_1 = filesystem::remove_base(&current_dir, &self.1);
        let location_2 = filesystem::remove_base(&current_dir, &self.3);
        let pos_1 = &self.2;
        let pos_2 = &self.4;
        write!(f, "duplicate primary design units identified as '{}'\n\nlocation 1: {}:{}\nlocation 2: {}:{}\n\n{}", self.0, filesystem::normalize_path(location_1).display(), pos_1, filesystem::normalize_path(location_2).display(), pos_2, HINT)
    }
}

const HINT: &str = "hint: To resolve this error either
    1) rename one of the units to a unique identifier
    2) add one of the file paths to a .orbitignore file";