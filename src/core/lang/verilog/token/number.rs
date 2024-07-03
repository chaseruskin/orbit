use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Decimal(String),
    Based(String),
    Real(String),
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Decimal(s) => s.to_string(),
                Self::Based(b) => b.to_string(),
                Self::Real(r) => r.to_string(),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum BaseSpec {
    Decimal(char),
    Hexadecimal(char),
    Octal(char),
    Binary(char),
}
