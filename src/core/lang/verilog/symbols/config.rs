use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    reference::{CompoundIdentifier, RefSet},
    sv::token::{keyword::Keyword, operator::Operator, token::SystemVerilogToken},
    verilog::{error::VerilogError, token::identifier::Identifier},
};

#[derive(Debug, PartialEq)]
pub struct Config {
    name: Identifier,
    refs: RefSet,
    pos: Position,
}

impl Config {
    pub fn get_name(&self) -> &Identifier {
        &self.name
    }

    pub fn get_position(&self) -> &Position {
        &self.pos
    }

    pub fn get_refs(&self) -> &RefSet {
        &self.refs
    }
}

impl Config {
    pub fn from_tokens<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take config name
        let config_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(VerilogError::Vague),
        };

        // take terminator ';'
        tokens.next().take().unwrap();

        let mut refs = RefSet::new();

        // parse until finding `endconfig`
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endconfig));
            } else if t.as_type().check_keyword(&Keyword::Endconfig) {
                // exit the loop for parsing the config
                break;
            // parse to try to find a module name
            } else if t.as_type().check_keyword(&Keyword::Design) {
                refs.extend(Self::parse_design_statement(tokens)?);
            }
        }

        Ok(Config {
            name: config_name,
            refs: refs,
            pos: pos,
        })
    }

    fn parse_design_statement<I>(tokens: &mut Peekable<I>) -> Result<RefSet, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut refs = RefSet::new();
        while let Some(t) = tokens.next() {
            if t.as_type().check_delimiter(&Operator::Terminator) {
                break;
            } else if let Some(name) = t.take().take_identifier() {
                // skip the library name if this is what we are looking at
                if let Some(t_next) = tokens.peek() {
                    if t_next.as_type().check_delimiter(&Operator::Dot) == true {
                        continue;
                    }
                }
                refs.insert(CompoundIdentifier::new_minimal_verilog(name));
            }
        }
        Ok(refs)
    }
}
