use super::super::sv::token::{keyword::Keyword, operator::Operator};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VerilogError {
    #[error("an error has occurred.")]
    Unknown,
    #[error("missing closing sequence for block comment (*/)")]
    UnclosedBlockComment,
    #[error("invalid character {0}")]
    InvalidChar(char),
    #[error("invalid sequence {0}")]
    InvalidSequence(String),
    #[error("expecting closing delimiter {0}")]
    UnclosedLiteral(char),
    #[error("expecting +, -, or digit, but got {0}")]
    InvalidExponChar(char),
    #[error("expecting +, -, or digit, but got nothing")]
    EmptyExponChar,
    #[error("expecting digits for exponent value but got nothing")]
    EmptyExponNumber,
    #[error("expecting numeric value after base specifier")]
    EmptyBaseConstNumber,
    #[error("expecting base specifier for based constant")]
    MissingBaseSpecifier,
    #[error("invalid base specifier {0}")]
    InvalidBaseSpecifier(char),
    #[error("invalid character {0} after digit")]
    InvalidCharInNumber(char),
    #[error("missing digits after decimal point")]
    MissingNumbersAfterDecimalPoint,
    #[error("expecting keyword or identifier immediately after compiler directive `")]
    EmptyCompilerDirective,
    #[error("invalid syntax")]
    Vague,
    #[error("expecting \"{0}\" keyword")]
    ExpectingKeyword(Keyword),
    #[error("expecting \"{0}\" delimiter")]
    ExpectingOperator(Operator),
    #[error("incomplete module declaration")]
    ModDecIncomplete,
}
