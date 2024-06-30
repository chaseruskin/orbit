use super::{Identifier, Position};
use crate::core::lang::reference::RefSet;
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(transparent)]
pub struct Architecture {
    name: Identifier,
    #[serde(skip_serializing)]
    owner: Identifier,
    #[serde(skip_serializing)]
    refs: RefSet,
    #[serde(skip_serializing)]
    deps: RefSet,
    #[serde(skip_serializing)]
    pos: Position,
}

impl Architecture {
    pub fn new(
        name: Identifier,
        owner: Identifier,
        refs: RefSet,
        deps: RefSet,
        pos: Position,
    ) -> Self {
        Self {
            name: name,
            owner: owner,
            refs: refs,
            deps: deps,
            pos: pos,
        }
    }
}

impl Architecture {
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
        &self.refs
    }

    pub fn get_deps(&self) -> &RefSet {
        &self.deps
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
