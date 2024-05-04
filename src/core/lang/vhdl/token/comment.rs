use super::super::super::lexer::Position;
use super::super::token::char_set;
use crate::core::lang::vhdl::token::ToColor;
use colored::ColoredString;
use colored::Colorize;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Comment {
    Single(String),
    Delimited(String),
}

impl Comment {
    fn as_str(&self) -> &str {
        match self {
            Self::Single(note) => note.as_ref(),
            Self::Delimited(note) => note.as_ref(),
        }
    }

    /// Computes the ending position the cursor ends up in.
    pub fn ending_position(&self) -> Position {
        // begin with counting the opening delimiters (-- or /*)
        let mut pos = Position::place(1, 2);
        let mut chars = self.as_str().chars();
        while let Some(c) = chars.next() {
            if char_set::is_newline(&c) == true {
                pos.next_line();
            } else {
                pos.next_col();
            }
        }
        match self {
            Self::Single(_) => (),
            // increment to handle the closing delimiters */
            Self::Delimited(_) => {
                pos.next_col();
                pos.next_col();
            }
        }
        pos
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(c) => write!(f, "--{}", c),
            Self::Delimited(c) => write!(f, "/*{}*/", c),
        }
    }
}

impl ToColor for Comment {
    fn to_color(&self) -> ColoredString {
        self.to_string().green()
    }
}
