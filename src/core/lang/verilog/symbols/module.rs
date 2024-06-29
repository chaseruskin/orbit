use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    verilog::{
        error::VerilogError,
        token::{identifier::Identifier, token::VerilogToken},
    },
};

use super::VerilogSymbol;

#[derive(Debug, PartialEq)]
pub struct Module {
    name: Identifier,
    parameters: Vec<String>,
    ports: Vec<String>,
    refs: Vec<Identifier>,
    pos: Position,
    language: String,
}

impl Module {
    pub fn get_name(&self) -> &Identifier {
        &self.name
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
        println!("{:?}", mod_name);
        let mut refs = Vec::new();
        // parse the interface/declaration of the module
        let (parameters, ports, d_refs) = VerilogSymbol::parse_module_declaration(tokens)?;
        refs.extend(d_refs);
        // parse the body of the module
        let (body_parameters, body_ports, b_refs) =
            VerilogSymbol::parse_module_architecture(tokens)?;
        refs.extend(b_refs);

        let parameters = parameters
            .into_iter()
            .map(|f| f.0)
            .collect::<Vec<Vec<Token<VerilogToken>>>>();

        let ports = ports
            .into_iter()
            .map(|f| f.0)
            .collect::<Vec<Vec<Token<VerilogToken>>>>();

        Ok(Module {
            name: match mod_name {
                VerilogToken::Identifier(id) => id,
                // expecting identifier
                _ => return Err(VerilogError::Vague),
            },
            parameters: Vec::new(),
            ports: Vec::new(),
            refs: refs,
            pos: pos,
            language: String::from("verilog"),
        })
    }
}
