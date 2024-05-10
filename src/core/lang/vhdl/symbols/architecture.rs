use super::{Identifier, IdentifierList, Position};
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(transparent)]
pub struct Architecture {
    name: Identifier,
    #[serde(skip_serializing)]
    owner: Identifier,
    #[serde(skip_serializing)]
    dependencies: IdentifierList,
    #[serde(skip_serializing)]
    refs: IdentifierList,
    #[serde(skip_serializing)]
    pos: Position,
}

impl Architecture {
    pub fn new(
        name: Identifier,
        owner: Identifier,
        deps: IdentifierList,
        refs: IdentifierList,
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

    pub fn edges(&self) -> &IdentifierList {
        &self.dependencies
    }

    pub fn get_deps(&self) -> &IdentifierList {
        &self.dependencies
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &IdentifierList {
        &self.refs
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut IdentifierList {
        &mut self.refs
    }
}
