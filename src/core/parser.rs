use crate::core::lexer::Token;
use std::fmt::Display;

pub trait Parse<T> {
    type SymbolType;
    type Err;

    fn parse(tokens: Vec<Token<T>>) -> Vec<Result<Symbol<Self::SymbolType>, SymbolError<Self::Err>>> 
        where <Self as Parse<T>>::Err: Display;
}

#[derive(Debug, PartialEq)]
pub struct Symbol<T> {
    stype: T,
}

impl<T> Symbol<T> {
    /// Creates a new symbol.
    pub fn new(stype: T) -> Self {
        Self {
            stype: stype,
        }
    }

    pub fn take(self) -> T {
        self.stype
    }

    pub fn as_ref(&self) -> &T {
        &self.stype
    }
}

#[derive(Debug, PartialEq)]
pub struct SymbolError<T: Display> {
    err: T,
}

impl<T: Display> SymbolError<T> {
    /// Creates a new `SymbolError` struct at position `loc` with error `T`.
    pub fn new(err: T) -> Self {
        Self { err: err }
    }
}

impl<T: Display> Display for SymbolError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}