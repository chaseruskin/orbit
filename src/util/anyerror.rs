use std::error::Error;
use std::fmt::Display;

/// Quickly implement a custom/unique error message.
///
/// Can also be used to wrap an error's message.
#[derive(Debug, PartialEq)]
pub struct AnyError(pub String);

impl Error for AnyError {}

impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Fault> for AnyError {
    fn from(value: Fault) -> Self {
        AnyError(value.to_string())
    }
}

impl From<&str> for AnyError {
    fn from(value: &str) -> Self {
        AnyError(value.to_string())
    }
}

pub type Fault = Box<dyn Error>;
