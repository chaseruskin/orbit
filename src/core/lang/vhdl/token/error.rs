use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum VHDLTokenError {
    #[error("{0}")]
    Any(String),
    #[error("invalid character {0}")]
    Invalid(String),
    #[error("Missing and empty {0}")]
    MissingAndEmpty(char),
    #[error("Expecting closing {0} but got {1}")]
    MissingClosingAndGot(char, char),
}
