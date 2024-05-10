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

#[derive(Debug, thiserror::Error)]
pub struct CodeFault(pub Option<String>, pub Fault);

impl From<Fault> for CodeFault {
    fn from(value: Fault) -> Self {
        Self(None, value)
    }
}

impl CodeFault {
    /// Checks if there is a source code parsing error.
    pub fn is_source_err(&self) -> bool {
        self.0.is_some()
    }

    /// References the source code file that produced an error, it exists.
    pub fn as_source_file(&self) -> Option<&String> {
        self.0.as_ref()
    }

    pub fn into_fault(self) -> Fault {
        self.1
    }
}

impl Display for CodeFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}