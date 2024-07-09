use crate::core::lang::{
    lexer::{self, Tokenize},
    sv::error::SystemVerilogError,
};

use super::token::SystemVerilogToken;

#[derive(Debug, PartialEq)]
struct SystemVerilogElement(
    Result<lexer::Token<SystemVerilogToken>, lexer::TokenError<SystemVerilogError>>,
);

#[derive(PartialEq)]
pub struct SystemVerilogTokenizer {
    tokens: Vec<SystemVerilogElement>,
}

impl Tokenize for SystemVerilogTokenizer {
    type TokenType = SystemVerilogToken;
    type Err = SystemVerilogError;

    fn tokenize(s: &str) -> Vec<Result<lexer::Token<Self::TokenType>, lexer::TokenError<Self::Err>>>
    where
        <Self as Tokenize>::Err: std::fmt::Display,
    {
        todo!()
    }
}
