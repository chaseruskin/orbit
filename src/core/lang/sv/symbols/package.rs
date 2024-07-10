use std::iter::Peekable;

use crate::core::lang::{
    lexer::{Position, Token},
    reference::RefSet,
    sv::{
        error::SystemVerilogError,
        token::{identifier::Identifier, keyword::Keyword, token::SystemVerilogToken},
    },
};

#[derive(Debug, PartialEq)]
pub struct Package {
    name: Identifier,
    refs: RefSet,
    pos: Position,
}

impl Package {
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

impl Package {
    pub fn from_tokens<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // take config name
        let config_name = match tokens.next().take().unwrap().take() {
            SystemVerilogToken::Identifier(id) => id,
            _ => return Err(SystemVerilogError::Vague),
        };

        // take terminator ';'
        tokens.next().take().unwrap();

        let refs = RefSet::new();

        // parse until finding `endconfig`
        while let Some(t) = tokens.next() {
            if t.as_type().is_eof() == true {
                return Err(SystemVerilogError::ExpectingKeyword(Keyword::Endpackage));
            } else if t.as_type().check_keyword(&Keyword::Endpackage) {
                // exit the loop for parsing the config
                break;
            // parse other references
            } else if t.as_type().check_keyword(&Keyword::Import) {
                // TODO
            }
        }

        Ok(Package {
            name: config_name,
            refs: refs,
            pos: pos,
        })
    }
}
