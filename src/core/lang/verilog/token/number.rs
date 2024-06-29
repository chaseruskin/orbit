#[derive(Debug, PartialEq)]
pub enum Number {
    Decimal(String),
    Based(String),
    Real(String),
}

#[derive(Debug, PartialEq)]
pub enum BaseSpec {
    Decimal(char),
    Hexadecimal(char),
    Octal(char),
    Binary(char),
}
