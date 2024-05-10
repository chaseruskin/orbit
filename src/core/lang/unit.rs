use super::vhdl::symbols::entity::Entity;

/// A unit describes the glue layer between HDL-specific symbol 'units': mainly entities
/// and modules.

#[derive(Debug, PartialEq)]
pub enum Unit {
    Vhdl(Entity),
    Verilog(String),
}

impl Unit {
    pub fn into_vhdl_entity(self) -> Entity {
        match self {
            Self::Vhdl(e) => e,
            _ => panic!("conversion to vhdl entity is currently unsupported"),
        }
    }

    pub fn into_verilog_module(self) -> String {
        match self {
            Self::Verilog(m) => m,
            _ => panic!("conversion to verilog module is currently unsupported"),
        }
    }
}
