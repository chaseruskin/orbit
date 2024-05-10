use super::super::lexer::Position;
use super::symbols::VhdlSymbol;
use crate::core::ip::IpSpec;
use crate::core::lang::parser::ParseError;
use crate::core::lang::vhdl::symbols::VHDLParser;
use crate::core::lang::vhdl::token::identifier::Identifier;
use crate::util::anyerror::CodeFault;
use crate::util::filesystem;
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use toml_edit::InlineTable;

pub type PrimaryUnitStore = HashMap<Identifier, PrimaryUnit>;

#[derive(PartialEq, Hash, Eq, Debug)]
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
        tbl.insert(
            "identifier",
            toml_edit::value(&self.get_iden().to_string())
                .into_value()
                .unwrap(),
        );
        tbl.insert(
            "type",
            toml_edit::value(&self.to_string()).into_value().unwrap(),
        );
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
        write!(
            f,
            "{}",
            match self {
                Self::Entity(_) => "entity",
                Self::Package(_) => "package",
                Self::Context(_) => "context",
                Self::Configuration(_) => "configuration",
            }
        )
    }
}

#[derive(Debug)]
pub struct Unit {
    name: Identifier,
    symbol: Option<VhdlSymbol>,
    /// source code file
    source: String,
}

impl Unit {
    pub fn get_symbol(&self) -> Option<&VhdlSymbol> {
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

pub fn collect_units(files: &Vec<String>) -> Result<HashMap<Identifier, PrimaryUnit>, CodeFault> {
    let mut result: HashMap<Identifier, PrimaryUnit> = HashMap::new();
    // iterate through all source files
    for source_file in files {
        // only read the HDL files
        if crate::core::fileset::is_vhdl(&source_file) == true {
            // parse text into VHDL symbols
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = match VHDLParser::read(&contents) {
                Ok(s) => s.into_symbols(),
                Err(e) => Err(CodeFault(
                    Some(source_file.clone()),
                    Box::new(ParseError::SourceCodeError(
                        source_file.clone(),
                        e.to_string(),
                    )),
                ))?,
            };
            // transform into primary design units
            let units: Vec<PrimaryUnit> = symbols
                .into_iter()
                .filter_map(|sym| {
                    let name = sym.as_iden()?.clone();
                    match sym {
                        VhdlSymbol::Entity(_) => Some(PrimaryUnit::Entity(Unit {
                            name: name,
                            symbol: Some(sym),
                            source: source_file.clone(),
                        })),
                        VhdlSymbol::Package(_) => Some(PrimaryUnit::Package(Unit {
                            name: name,
                            symbol: Some(sym),
                            source: source_file.clone(),
                        })),
                        VhdlSymbol::Configuration(_) => Some(PrimaryUnit::Configuration(Unit {
                            name: name,
                            symbol: Some(sym),
                            source: source_file.clone(),
                        })),
                        VhdlSymbol::Context(_) => Some(PrimaryUnit::Context(Unit {
                            name: name,
                            symbol: Some(sym),
                            source: source_file.clone(),
                        })),
                        _ => None,
                    }
                })
                .collect();

            for primary in units {
                if let Some(dupe) = result.insert(primary.get_iden().clone(), primary) {
                    return Err(CodeFault(
                        None,
                        Box::new(VhdlIdentifierError::DuplicateIdentifier(
                            dupe.get_iden().to_string(),
                            PathBuf::from(source_file),
                            result
                                .get(dupe.get_iden())
                                .unwrap()
                                .get_unit()
                                .get_symbol()
                                .unwrap()
                                .get_position()
                                .clone(),
                            PathBuf::from(dupe.get_unit().get_source_code_file()),
                            dupe.get_unit().get_symbol().unwrap().get_position().clone(),
                        )),
                    ))?;
                }
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]
pub enum VhdlIdentifierError {
    DuplicateIdentifier(String, PathBuf, Position, PathBuf, Position),
    DuplicateAcrossDirect(String, IpSpec, PathBuf, Position),
}

impl std::error::Error for VhdlIdentifierError {}

impl std::fmt::Display for VhdlIdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateIdentifier(iden, path1, loc1, path2, loc2) => {
                let current_dir = std::env::current_dir().unwrap();
                let location_1 = filesystem::remove_base(&current_dir, &path1);
                let location_2 = filesystem::remove_base(&current_dir, &path2);
                write!(f, "duplicate primary design units identified as '{}'\n\nlocation 1: {}{}\nlocation 2: {}{}\n\n{}", 
                    iden,
                    filesystem::into_std_str(location_1), loc1,
                    filesystem::into_std_str(location_2), loc2,
                    HINT)
            }
            Self::DuplicateAcrossDirect(iden, dep, path, pos) => {
                let current_dir = std::env::current_dir().unwrap();
                let location = filesystem::remove_base(&current_dir, &path);
                write!(f, "duplicate primary design units identified as '{}'\n\nlocation: {}{}\nconflicts with direct dependency {}\n\n{}", 
                iden,
                filesystem::into_std_str(location), pos,
                dep,
                HINT_2)
            }
        }
    }
}

const HINT: &str = "hint: To resolve this error either
    1) rename one of the units to a unique identifier
    2) add one of the file paths to a .orbitignore file";

const HINT_2: &str = "hint: To resolve this error either
    1) rename the unit in the current ip to a unique identifier
    2) remove the direct dependency from Orbit.toml
    3) add the file path for the unit from the current ip to a .orbitignore file";
