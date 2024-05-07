use super::{
    symbols::{self, IdentifierList},
    token::identifier::Identifier,
};

#[derive(Debug, PartialEq)]
pub enum SubUnit {
    Configuration(symbols::configuration::Configuration),
    Architecture(symbols::architecture::Architecture),
}

impl SubUnit {
    pub fn from_arch(arch: symbols::architecture::Architecture) -> Self {
        Self::Architecture(arch)
    }

    pub fn from_config(cfg: symbols::configuration::Configuration) -> Self {
        Self::Configuration(cfg)
    }

    pub fn get_edges(&self) -> &IdentifierList {
        match self {
            Self::Architecture(u) => u.edges(),
            Self::Configuration(u) => u.edges(),
        }
    }

    pub fn get_entity(&self) -> &Identifier {
        match self {
            Self::Architecture(u) => u.entity(),
            Self::Configuration(u) => u.entity(),
        }
    }

    pub fn get_refs(&self) -> &IdentifierList {
        match self {
            Self::Architecture(u) => u.get_refs(),
            Self::Configuration(u) => u.get_refs(),
        }
    }
}
