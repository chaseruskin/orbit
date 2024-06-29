use serde_derive::Serialize;

use super::super::error::VerilogError;
use super::token::VerilogToken;
use super::tokenizer::char_set;
use crate::core::lang::lexer::TrainCar;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::str::FromStr;

#[derive(Debug, Clone, PartialOrd, Ord, Serialize)]
pub enum Identifier {
    Basic(String),
    Escaped(String),
    System(String),
}

impl Eq for Identifier {}

impl Identifier {
    // Returns the reference to the inner `String` struct.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Basic(id) => id.as_ref(),
            Self::Escaped(id) => id.as_ref(),
            Self::System(id) => id.as_ref(),
        }
    }

    /// Checks if the identifier is a system task/function.
    fn is_system(&self) -> bool {
        match self {
            Self::System(_) => true,
            _ => false,
        }
    }
}

// TODO: test
impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Basic(id) => id.to_lowercase().hash(state),
            Self::Escaped(id) => id.hash(state),
            Self::System(id) => id.hash(state),
        }
    }
}

impl FromStr for Identifier {
    type Err = VerilogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = TrainCar::new(s.chars());
        match chars.consume() {
            // check what type of identifier it is
            Some(c) => Ok(match c {
                '\\' => Self::Escaped(VerilogToken::consume_value_pattern(
                    &mut chars,
                    None,
                    char_set::is_not_whitespace,
                )?),
                '$' => Self::System(VerilogToken::consume_value_pattern(
                    &mut chars,
                    None,
                    char_set::is_identifier_character,
                )?),
                _ => {
                    // verify the first character was a letter or underscore
                    match char_set::is_letter(&c) || c == char_set::UNDER_SCORE {
                        true => Self::Basic(VerilogToken::consume_value_pattern(
                            &mut chars,
                            Some(c),
                            char_set::is_identifier_character,
                        )?),
                        false => return Err(VerilogError::InvalidChar(c)),
                    }
                }
            }),
            None => panic!("No more characters"),
        }
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // compare the two strings
        if self.is_system() == true && other.is_system() == true {
            self.as_str() == other.as_str()
        } else if self.is_system() == true || other.is_system() == true {
            false
        } else {
            self.as_str() == other.as_str()
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(id) => write!(f, "{}", id),
            Self::Escaped(id) => write!(f, "\\{}", id),
            Self::System(id) => write!(f, "${}", id),
        }
    }
}
