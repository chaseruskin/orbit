use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    reference::{CompoundIdentifier, RefSet},
    verilog::{
        error::VerilogError,
        interface::{self, ParamList, PortList},
        token::{identifier::Identifier, operator::Operator, token::VerilogToken},
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
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_deps(&self) -> &RefSet {
        &self.deps
    }

    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }

    pub fn get_refs(&self) -> &RefSet {
        &self.refs
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

    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        // take module name
        let mod_name = tokens.next().take().unwrap().take();
        // println!("{:?}", mod_name);
        let mut refs = RefSet::new();
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
        // println!("{}", mod_name);
        // println!("{:?}", ports);
        // println!("{:?}", params);
        Ok(Module {
            name: match mod_name {
                VerilogToken::Identifier(id) => id,
                // expecting identifier
                _ => return Err(VerilogError::Vague),
            },
            parameters: params,
            ports: ports,
            refs: refs,
            deps: deps,
            pos: pos,
            language: String::from("verilog"),
        })
    }
}
