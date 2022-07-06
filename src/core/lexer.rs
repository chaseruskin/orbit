use std::iter::Peekable;
use std::fmt::Display;

pub trait Tokenize {
    type TokenType;
    type Err;

    fn tokenize(s: &str) -> Vec<Result<Token<Self::TokenType>, TokenError<Self::Err>>> 
        where <Self as Tokenize>::Err: Display;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<T> {
    position: Position,
    ttype: T,
}

impl<T> Token<T> {
    pub fn as_type(&self) -> &T {
        &self.ttype
    }

    /// Transforms the token into its type.
    pub fn take(self) -> T {
        self.ttype
    }

    /// Returns the position in the file where the token was captured.
    pub fn locate(&self) -> &Position {
        &self.position
    }

    /// Creates a new token.
    pub fn new(ttype: T, loc: Position) -> Self {
        Self {
            position: loc,
            ttype: ttype,
        }
    }

    /// References the inner token type.
    pub fn as_ref(&self) -> &T {
        &self.ttype
    }
}

#[derive(Debug, PartialEq)]
pub struct TokenError<T: Display> {
    position: Position,
    err: T,
}

impl<T: Display> TokenError<T> {
    /// Creates a new `TokenError` struct at position `loc` with error `T`.
    pub fn new(err: T, loc: Position) -> Self {
        Self {
            position: loc,
            err: err
        }
    }
}

impl<T: Display> Display for TokenError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.err)
    }
}

#[derive(Debug, PartialEq, Clone)]
/// (Line, Col)
pub struct Position(usize, usize);

impl Position {
    /// Creates a new `Position` struct as line 1, col 0.
    pub fn new() -> Self {
        Position(1, 0)
    }

    /// Creates a `Position` struct at a particular location `line`:`col`.
    pub fn place(line: usize, col: usize) -> Self {
        Self(line, col)
    }

    /// Increments the column counter by 1.
    pub fn next_col(&mut self) {
        self.1 += 1;
    }   

    /// Increments the column counter by 1. If the current char `c` is a newline,
    /// it will then drop down to the next line.
    pub fn step(&mut self, c: &char) {
        self.next_col();
        if c == &'\n' {
            self.next_line();
        }
    }

    /// Increments the line counter by 1.
    /// 
    /// Also resets the column counter to 0.
    pub fn next_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Access the line (`.0`) number.
    pub fn line(&self) -> usize {
        self.0
    }

    /// Access the col (`.1`) number.
    pub fn col(&self) -> usize {
        self.1
    }

    /// Appends the position by adding lines and setting column.
    pub fn fast_forward(&mut self, other: &Position) {
        if other.0 > 1 {
            self.0 += other.0;
        }
        match other.0 > 1 {
            true => self.1 = other.1,   // set column
            false => self.1 += other.1, // add column
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

/// Helps keep the current position in the contents as the characters are consumed.
pub struct TrainCar<T> where T: Iterator<Item=char> {
    contents: Peekable<T>,
    loc: Position,
}

impl<T> TrainCar<T> where T: Iterator<Item=char> {
    /// Creates a new `TrainCar` struct with an initial position (1, 0) and a
    /// train `s`.
    pub fn new(s: T) -> Self {
        Self {
            loc: Position::new(),
            contents: s.peekable(),
        }
    }

    /// Takes the next char in the iterator and steps the `Position` marker 
    /// accordingly, if a char exists.
    pub fn consume(&mut self) -> Option<char> {
        if let Some(c) = self.contents.next() {
            self.loc.step(&c);
            Some(c)
        } else {
            None
        }
    }

    /// References the next char in the iterator, if it exists.
    pub fn peek(&mut self) -> Option<&char> {
        self.contents.peek()
    }

    /// Access the position of the first remainig character.
    pub fn locate(&self) -> &Position {
        &self.loc
    }

    /// References the entire iterator still remaining in `self`.
    pub fn peekable(&self) -> &Peekable<T> {
        &self.contents
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn moving_position() {
        let mut pos = Position::new();
        assert_eq!(pos, Position::place(1, 0));
        pos.next_col();
        assert_eq!(pos, Position::place(1, 1));
        pos.next_col();
        assert_eq!(pos, Position::place(1, 2));
        pos.next_line();
        assert_eq!(pos, Position::place(2, 0));
        pos.next_line();
        assert_eq!(pos, Position::place(3, 0));
    }
}