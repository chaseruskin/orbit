use crate::{core::lang::verilog::symbols::VerilogParser, util::anyerror::CodeFault};
use std::collections::HashMap;

use super::{symbols::VerilogSymbol, token::identifier::Identifier};

#[derive(PartialEq, Hash, Eq, Debug)]
pub enum PrimaryUnitType {
    Module,
}

#[derive(PartialEq, Hash, Eq, Debug)]
pub struct PrimaryUnit {
    dtype: PrimaryUnitType,
    unit: Unit,
}

impl PrimaryUnit {
    pub fn get_name(&self) -> &Identifier {
        &self.unit.name
    }

    pub fn get_unit(&self) -> &Unit {
        &self.unit
    }
}

impl std::fmt::Display for PrimaryUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.dtype {
                PrimaryUnitType::Module => "module",
            }
        )
    }
}

#[derive(Debug)]
pub struct Unit {
    name: Identifier,
    symbol: Option<VerilogSymbol>,
    /// source code file
    source: String,
}

impl Unit {
    pub fn get_symbol(&self) -> Option<&VerilogSymbol> {
        self.symbol.as_ref()
    }

    pub fn get_symbol_mut(&mut self) -> Option<&mut VerilogSymbol> {
        self.symbol.as_mut()
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
        if crate::core::fileset::is_verilog(&source_file) == true {
            println!("parse verilog: {:?}", source_file);
            // parse text into Verilog symbols
            // println!("Detected verilog file: {}", source_file);
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = match VerilogParser::read(&contents) {
                Ok(s) => s.into_symbols(),
                Err(e) => Err(CodeFault(Some(source_file.clone()), Box::new(e)))?,
            };

            // transform into primary design units
            let units: HashMap<Identifier, PrimaryUnit> = symbols
                .into_iter()
                .filter_map(|sym: VerilogSymbol| {
                    let name = sym.as_name().clone();
                    match sym {
                        VerilogSymbol::Module(_) => Some((
                            name.clone(),
                            PrimaryUnit {
                                dtype: PrimaryUnitType::Module,
                                unit: Unit {
                                    name: name,
                                    symbol: Some(sym),
                                    source: source_file.clone(),
                                },
                            },
                        )),
                    }
                })
                .collect();

            // push to the global list
            result.extend(units);
        }
    }
    Ok(result)
}
