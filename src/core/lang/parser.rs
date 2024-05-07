use super::lexer::Token;
use thiserror::Error;

pub trait Parse<T> {
    type SymbolType;
    type SymbolError;

    fn parse(
        tokens: Vec<Token<T>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>>;
}

#[derive(Debug, PartialEq)]
pub struct Symbol<T> {
    stype: T,
}

impl<T> Symbol<T> {
    /// Creates a new symbol.
    pub fn new(stype: T) -> Self {
        Self { stype: stype }
    }

    pub fn take(self) -> T {
        self.stype
    }

    pub fn as_ref(&self) -> &T {
        &self.stype
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum ParseError {
    #[error("file {0}: {1}")]
    SourceCodeError(String, String)
}