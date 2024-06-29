use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VerilogError {
    #[error("An error has occurred.")]
    Unknown,
    #[error("Missing closing sequence for block comment (*/)")]
    UnclosedBlockComment,
    #[error("Invalid character {0}")]
    InvalidChar(char),
    #[error("Invalid sequence {0}")]
    InvalidSequence(String),
    #[error("Expecting closing delimiter {0}")]
    UnclosedLiteral(char),
    #[error("Expecting +, -, or digit, but got {0}")]
    InvalidExponChar(char),
    #[error("Expecting +, -, or digit, but got nothing")]
    EmptyExponChar,
    #[error("Expecting digits for exponent value but got nothing")]
    EmptyExponNumber,
    #[error("Expecting numeric value after base specifier")]
    EmptyBaseConstNumber,
    #[error("Expecting base specifier for based constant")]
    MissingBaseSpecifier,
    #[error("Invalid base specifier {0}")]
    InvalidBaseSpecifier(char),
    #[error("Invalid character {0} after digit")]
    InvalidCharInNumber(char),
    #[error("Missing digits after decimal point")]
    MissingNumbersAfterDecimalPoint,
    #[error("Expecting keyword or identifier immediately after compiler directive `")]
    EmptyCompilerDirective,
    #[error("invalid syntax")]
    Vague,
}
