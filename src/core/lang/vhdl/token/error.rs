use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum VhdlError {
    #[error("{0}")]
    Any(String),
    #[error("invalid character {0}")]
    Invalid(String),
    #[error("missing and empty {0}")]
    MissingAndEmpty(char),
    #[error("expecting closing {0} but got {1}")]
    MissingClosingAndGot(char, char),
    #[error("invalid source code")]
    Vague
}
