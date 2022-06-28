/// Quickly implement a custom/unique error message. 
/// 
/// Can also be used to wrap an error's message.
#[derive(Debug, PartialEq)]
pub struct AnyError(pub String);

impl std::error::Error for AnyError {}

impl std::fmt::Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Fault = Box<dyn std::error::Error>;