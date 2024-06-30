use std::fmt::Display;

use crate::core::lang::reference::RefSet;

use super::{packagebody::PackageBody, Generics, Identifier, Position};

#[derive(Debug, PartialEq)]
pub struct Package {
    name: Identifier,
    generics: Generics,
    body: Option<PackageBody>,
    refs: RefSet,
    pos: Position,
}

impl Package {
    pub fn new(name: Identifier, refs: RefSet, pos: Position) -> Self {
        Self {
            name: name,
            generics: Generics::new(),
            body: None,
            refs: refs,
            pos: pos,
        }
    }

    pub fn generics(mut self, generics: Generics) -> Self {
        self.generics = generics;
        self
    }

    pub fn body(mut self, body: Option<PackageBody>) -> Self {
        self.body = body;
        self
    }
}

impl Package {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    /// Accesses the references for the entity.
    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    /// Accesses the references as mutable for the entity.
    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        &mut self.refs
    }

    pub fn into_refs(self) -> RefSet {
        self.refs
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
