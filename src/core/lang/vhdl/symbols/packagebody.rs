use std::fmt::Display;

use super::{Identifier, IdentifierList, Position};

#[derive(Debug, PartialEq)]
pub struct PackageBody {
    owner: Identifier,
    refs: IdentifierList,
    pos: Position,
}

impl PackageBody {
    pub fn new(owner: Identifier, refs: IdentifierList, pos: Position) -> Self {
        Self {
            owner: owner,
            refs: refs,
            pos: pos,
        }
    }
}

impl PackageBody {
    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &IdentifierList {
        &self.refs
    }

    pub fn get_owner(&self) -> &Identifier {
        &self.owner
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn take_refs(self) -> IdentifierList {
        self.refs
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut IdentifierList {
        &mut self.refs
    }

    pub fn into_refs(self) -> IdentifierList {
        self.refs
    }
}

impl Display for PackageBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package body for {}", self.owner)
    }
}
