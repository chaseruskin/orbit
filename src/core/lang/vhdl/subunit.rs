//
//  Copyright (C) 2022-2026  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use crate::core::lang::reference::{CompoundIdentifier, RefSet};

use super::{symbols, token::identifier::Identifier};

#[derive(Debug, PartialEq, Clone)]
pub enum SubUnit {
    Configuration(symbols::configuration::Configuration, String),
    Architecture(symbols::architecture::Architecture, String),
    PackageBody(symbols::packagebody::PackageBody, String),
}

impl SubUnit {
    pub fn from_arch(arch: symbols::architecture::Architecture, src: String) -> Self {
        Self::Architecture(arch, src)
    }

    pub fn from_config(cfg: symbols::configuration::Configuration, src: String) -> Self {
        Self::Configuration(cfg, src)
    }

    pub fn from_body(body: symbols::packagebody::PackageBody, src: String) -> Self {
        Self::PackageBody(body, src)
    }

    pub fn get_source_file(&self) -> &String {
        match self {
            Self::Architecture(_, s) => s,
            Self::Configuration(_, s) => s,
            Self::PackageBody(_, s) => s,
        }
    }

    pub fn into_refs(self) -> RefSet {
        match self {
            Self::Architecture(u, _) => u.into_refs(),
            Self::Configuration(u, _) => u.into_refs(),
            Self::PackageBody(u, _) => u.into_refs(),
        }
    }

    pub fn is_arch(&self) -> bool {
        match &self {
            Self::Architecture(_, _) => true,
            _ => false,
        }
    }

    pub fn into_arch(self) -> Option<symbols::architecture::Architecture> {
        match self {
            Self::Architecture(a, _) => Some(a),
            _ => None,
        }
    }

    /// Returns an ordered list of compound indentifiers for consist graph building.
    pub fn get_edge_list(&self) -> Vec<&CompoundIdentifier> {
        let mut list = Vec::with_capacity(self.get_refs().len());
        self.get_refs().iter().for_each(|f| {
            list.push(f);
        });
        list.extend(self.get_edge_list_entities());
        list.sort();
        list
    }

    /// Returns the list of compound identifiers that were parsed from entity instantiations.
    pub fn get_edge_list_entities(&self) -> Vec<&CompoundIdentifier> {
        let mut list = match self {
            Self::Architecture(arch, _) => arch.get_deps().into_iter().collect(),
            _ => Vec::new(),
        };
        list.sort();
        list
    }

    pub fn get_entity(&self) -> &Identifier {
        match self {
            Self::Architecture(u, _) => u.entity(),
            Self::Configuration(u, _) => u.entity(),
            Self::PackageBody(u, _) => u.get_owner(),
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match self {
            Self::Architecture(u, _) => u.get_refs(),
            Self::Configuration(u, _) => u.get_refs(),
            Self::PackageBody(u, _) => u.get_refs(),
        }
    }

    pub fn get_refs_mut(&mut self) -> &mut RefSet {
        match self {
            Self::Architecture(u, _) => u.get_refs_mut(),
            Self::Configuration(u, _) => u.get_refs_mut(),
            Self::PackageBody(u, _) => u.get_refs_mut(),
        }
    }
}
