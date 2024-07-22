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

use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    reference::{CompoundIdentifier, RefSet},
    sv::{
        symbols::SystemVerilogSymbol,
        token::{keyword::Keyword, token::SystemVerilogToken},
    },
    verilog::{
        error::VerilogError,
        interface::{self, ParamList, PortList},
        token::{identifier::Identifier, operator::Operator},
    },
};

use super::super::super::vhdl::token::Identifier as VhdlIdentifier;

use super::VerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Module {
    name: Identifier,
    parameters: ParamList,
    ports: PortList,
    refs: RefSet,
    deps: RefSet,
    pos: Position,
    language: String,
}

impl Module {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }

    pub fn extend_refs(&mut self, refs: RefSet) {
        self.refs.extend(refs);
    }
}

impl Module {
    pub fn into_declaration(&self) -> String {
        let mut result = String::new();

        result.push_str(&format!("module "));
        result.push_str(&self.name.to_string());
        result.push_str(&interface::display_interface(&self.parameters, true));
        result.push_str(&interface::display_interface(&self.ports, false));
        result.push(';');
        result
    }

    pub fn into_instance(
        &self,
        name: &Option<VhdlIdentifier>,
        signal_prefix: &str,
        signal_suffix: &str,
    ) -> String {
        let mut result = String::new();
        // module name
        result.push_str(&self.name.to_string());
        // parameters
        result.push_str(&interface::display_connections(
            &self.parameters,
            true,
            "",
            "",
        ));
        // leave whitespace between module name and instance if no parameters are available
        if self.parameters.is_empty() == true {
            result.push(' ');
        }

        // instance name
        if let Some(n) = name {
            result.push_str(&n.to_string());
        } else {
            result.push_str("uX");
        }

        // ports
        result.push_str(&interface::display_connections(
            &self.ports,
            false,
            signal_prefix,
            signal_suffix,
        ));
        result.push(';');
        result
    }

    pub fn into_wires(&self, wire_prefix: &str, wire_suffix: &str) -> String {
        let mut result = String::new();
        self.parameters.iter().for_each(|p| {
            result.push_str(&&&p.into_declaration(false, true, "", ""));
            result.push_str(&Operator::Terminator.to_string());
            result.push('\n');
        });
        if self.parameters.is_empty() == false {
            result.push('\n');
        }
        self.ports.iter().for_each(|p| {
            result.push_str(&&&p.into_declaration(false, false, wire_prefix, wire_suffix));
            result.push_str(&Operator::Terminator.to_string());
            result.push('\n');
        });
        result
    }
}

impl Module {
    pub fn get_deps(&self) -> &RefSet {
        &self.deps
    }

    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }
}

impl Module {
    /// Returns the list of compound identifiers that were parsed from entity instantiations.
    pub fn get_edge_list_entities(&self) -> Vec<CompoundIdentifier> {
        let mut list: Vec<CompoundIdentifier> = self.deps.iter().map(|f| f.clone()).collect();
        list.sort();
        list
    }

    pub fn get_edge_list(&self) -> Vec<CompoundIdentifier> {
        let mut list: Vec<CompoundIdentifier> = self.refs.iter().map(|f| f.clone()).collect();
        list.sort();
        list
    }

    /// Parses an `Module` design element from the module's identifier to
    /// the END closing statement.
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take module name
        let mod_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(VerilogError::Vague),
        };

        // initialize container for references to other design elements
        let mut refs = RefSet::new();

        // take all import statements
        while let Some(t) = tokens.peek() {
            if t.as_type().check_keyword(&Keyword::Import) {
                let _ = tokens.next().unwrap();
                let i_refs = SystemVerilogSymbol::parse_import_statement(tokens)?;
                refs.extend(i_refs);
            } else {
                break;
            }
        }

        // parse the interface/declaration of the module
        let (mut params, mut ports, d_refs) = VerilogSymbol::parse_module_declaration(tokens)?;
        refs.extend(d_refs);

        // parse the body of the module
        let (body_params, body_ports, b_refs, deps) =
            VerilogSymbol::parse_module_architecture(tokens)?;
        refs.extend(b_refs);

        // update declared ports from any architecture port definitions
        body_ports
            .into_iter()
            .for_each(|p| interface::update_port_list(&mut ports, p, false));

        // update declared params from any architecture param definitions
        body_params
            .into_iter()
            .for_each(|p| interface::update_port_list(&mut params, p, false));

        // for all ports and their datatypes, try to see if any are references to interfaces
        ports
            .iter()
            .filter_map(|p| p.as_user_defined_data_type())
            .for_each(|intf| {
                refs.insert(CompoundIdentifier::new_minimal_verilog(intf.clone()));
            });
        params
            .iter()
            .filter_map(|p| p.as_user_defined_data_type())
            .for_each(|intf| {
                refs.insert(CompoundIdentifier::new_minimal_verilog(intf.clone()));
            });

        // println!("{}", mod_name);
        // println!("{:?}", ports);
        // println!("{:?}", refs);
        // println!("{:?}", params);
        Ok(Module {
            name: mod_name,
            parameters: params,
            ports: ports,
            refs: refs,
            deps: deps,
            pos: pos,
            language: String::from("verilog"),
        })
    }
}
