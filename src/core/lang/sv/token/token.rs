use crate::core::lang::verilog::token::{identifier::Identifier, number::Number, token::Comment};

use super::{keyword::Keyword, operator::Operator};

#[derive(Debug, PartialEq, Clone)]
pub enum SystemVerilogToken {
    Comment(Comment),
    Operator(Operator),
    Number(Number),
    String(String),
    Identifier(Identifier),
    Keyword(Keyword),
    Directive(String),
    EOF,
}
