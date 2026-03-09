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

// This file is currently unused.

use crate::core::lang::verilog::symbols::module::Module;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::error::Error;
use serde_derive::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize)]
pub struct EntityJson<'a> {
    #[serde(flatten)]
    entity: &'a Entity,
    sources: &'a Vec<String>,
}

impl<'a> EntityJson<'a> {
    pub fn new(entity: &'a Entity, sources: &'a Vec<String>) -> Self {
        Self {
            entity: entity,
            sources: sources,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ModuleJson<'a> {
    #[serde(flatten)]
    module: &'a Module,
    sources: &'a Vec<String>,
}

impl<'a> ModuleJson<'a> {
    pub fn new(module: &'a Module, sources: &'a Vec<String>) -> Self {
        Self {
            module: module,
            sources: sources,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum UnitKey<'a> {
    Entity(EntityJson<'a>),
    Module(ModuleJson<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Legend<'a> {
    keys: Vec<UnitKey<'a>>,
}

impl<'a> Legend<'a> {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn add_module(&mut self, module: &'a Module, files: &'a Vec<String>) {
        self.keys
            .push(UnitKey::Module(ModuleJson::new(module, files)));
    }

    pub fn add_entity(&mut self, entity: &'a Entity, files: &'a Vec<String>) {
        self.keys
            .push(UnitKey::Entity(EntityJson::new(entity, files)));
    }

    pub fn get_filename(&self) -> String {
        String::from("legend.json")
    }

    pub fn get_keys(&self) -> &Vec<UnitKey<'a>> {
        &self.keys
    }

    pub fn write(&self, output_path: &PathBuf) -> Result<PathBuf, Error> {
        let legend_path = output_path.join(self.get_filename());
        let mut fd = File::create(&legend_path).expect("could not create legend file");

        let data = serde_json::to_string_pretty(self.get_keys()).unwrap();

        fd.write_all(data.as_bytes())
            .expect("failed to write data to legend");
        Ok(legend_path)
    }
}
