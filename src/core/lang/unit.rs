//
//  Copyright (C) 2022-2024  Chase Ruskin
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
