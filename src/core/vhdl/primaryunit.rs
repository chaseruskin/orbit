use crate::core::vhdl::token::Identifier;

use super::symbol::VHDLSymbol;
use crate::core::vhdl::symbol::VHDLParser;

#[derive(Debug, PartialEq)]
pub enum PrimaryUnit {
    Entity(Unit),
    Package(Unit),
    Context(Unit),
    Configuration(Unit),
}

#[derive(Debug, PartialEq)]
pub struct Unit {
    name: Identifier,
    file: String,
}


pub fn collect_units(files: &Vec<String>) -> Vec<PrimaryUnit> {
    let mut result = Vec::new();
    for source_file in files {
        if crate::core::fileset::is_vhdl(&source_file) == true {
            let contents = std::fs::read_to_string(&source_file).unwrap();
            let symbols = VHDLParser::read(&contents).into_symbols();
            // transform into primary design units
            let iter = symbols.into_iter().filter_map(|f| {
                let name = f.as_iden()?.clone();
                match f {
                    VHDLSymbol::Entity(_) => Some(PrimaryUnit::Entity(Unit{ name: name, file: source_file.clone() })),
                    VHDLSymbol::Package(_) => Some(PrimaryUnit::Package(Unit{ name: name, file: source_file.clone() })),
                    VHDLSymbol::Configuration(_) => Some(PrimaryUnit::Configuration(Unit{ name: name, file: source_file.clone() })),
                    VHDLSymbol::Context(_) => Some(PrimaryUnit::Context(Unit{ name: name, file: source_file.clone() })),
                    _ => None,
                }
            });
            result.append(&mut iter.collect());
        }
    }
    result
}