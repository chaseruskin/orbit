use crate::core::lang::reference::RefSet;

use super::{Identifier, Position};

#[derive(Debug, PartialEq)]
pub struct Configuration {
    name: Identifier,
    owner: Identifier,
    dependencies: RefSet,
    refs: RefSet,
    pos: Position,
}

impl Configuration {
    pub fn new(
        name: Identifier,
        owner: Identifier,
        deps: RefSet,
        refs: RefSet,
        pos: Position,
    ) -> Self {
        Self {
            name: name,
            owner: owner,
            dependencies: deps,
            refs: refs,
            pos: pos,
        }
    }
}

impl Configuration {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_owner(&self) -> &Identifier {
        &self.owner
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn entity(&self) -> &Identifier {
        &self.owner
    }

    pub fn edges(&self) -> &RefSet {
        &self.dependencies
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }

    pub fn into_refs(self) -> RefSet {
        self.refs
    }
}
