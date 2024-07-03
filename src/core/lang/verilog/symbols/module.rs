use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    reference::RefSet,
    verilog::{
        error::VerilogError,
        interface::PortList,
        token::{identifier::Identifier, token::VerilogToken},
    },
};

use super::VerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Module {
    name: Identifier,
    parameters: Vec<String>,
    ports: PortList,
    refs: RefSet,
    pos: Position,
    language: String,
}

impl Module {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }

    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }
}

impl Module {
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
        let (parameters, mut ports, d_refs) = VerilogSymbol::parse_module_declaration(tokens)?;
        refs.extend(d_refs);
        // parse the body of the module
        let (body_parameters, body_ports, b_refs) =
            VerilogSymbol::parse_module_architecture(tokens)?;
        refs.extend(b_refs);

        // TOOD: update declared ports from any architecture port definitions

        let parameters = parameters
            .into_iter()
            .map(|f| f)
            .collect::<Vec<Vec<Token<VerilogToken>>>>();

        Ok(Module {
            name: match mod_name {
                VerilogToken::Identifier(id) => id,
                // expecting identifier
                _ => return Err(VerilogError::Vague),
            },
            parameters: Vec::new(),
            ports: ports,
            refs: refs,
            pos: pos,
            language: String::from("verilog"),
        })
    }
}
