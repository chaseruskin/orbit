use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Decimal(String),
    Based(String),
    Real(String),
    Time(String),
}

impl Number {
    pub fn is_valid_time_units(s: &str) -> bool {
        match s {
            "s" | "ms" | "us" | "ns" | "ps" | "fs" => true,
            _ => false,
        }
    }
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
                Self::Time(t) => t.to_string(),
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
