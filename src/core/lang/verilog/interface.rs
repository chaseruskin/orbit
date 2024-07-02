use crate::core::lang::lexer::Token;

use super::{
    symbols::Statement,
    token::{identifier::Identifier, keyword::Keyword},
};

pub type PortList = Vec<Port>;

#[derive(Debug, PartialEq)]
pub enum Direction {
    Inout,
    Input,
    Output,
}

#[derive(Debug, PartialEq)]
pub struct Port {
    direction: Option<Keyword>,
    net_type: Option<Keyword>,
    is_reg: bool,
    is_signed: bool,
    range: Option<Statement>,
    name: Identifier,
}

impl Port {
    pub fn with(name: Identifier) -> Self {
        Self {
            direction: None,
            net_type: None,
            is_reg: false,
            is_signed: false,
            range: None,
            name: name,
        }
    }

    pub fn new() -> Self {
        Self {
            direction: None,
            net_type: None,
            is_reg: false,
            is_signed: false,
            range: None,
            name: Identifier::new(),
        }
    }

    pub fn inherit(&mut self, rhs: &Port) {
        self.direction = rhs.direction.clone();
        self.net_type = rhs.net_type.clone();
        self.is_reg = rhs.is_reg;
        self.is_signed = rhs.is_signed;
        if let Some(r) = &rhs.range {
            self.range = Some(
                r.iter()
                    .map(|f| Token::new(f.as_type().clone(), f.locate().clone()))
                    .collect(),
            )
        }
    }

    pub fn set_direction(&mut self, kw: Keyword) {
        self.direction = Some(kw);
    }

    pub fn set_net_type(&mut self, kw: Keyword) {
        self.net_type = Some(kw);
    }

    pub fn set_reg(&mut self) {
        self.is_reg = true;
    }

    pub fn set_signed(&mut self) {
        self.is_signed = true;
    }

    pub fn set_range(&mut self, stmt: Statement) {
        self.range = Some(stmt);
    }
}
