use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct Positional {
    name: String,
    short: Option<char>,
}

impl Positional {
    pub fn new(s: &str) -> Self {
        Positional { 
            name: s.to_string(),
            short: None,
        }
    }

    pub fn short(mut self, s: char) -> Self {
        self.short = Some(s);
        self
    }
}

impl Display for Positional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "<{}>", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Flag {
    name: String,
    short: Option<char>,
}

impl Flag {
    pub fn new(s: &str) -> Self {
        Flag { 
            name: s.to_string(),
            short: None,
        }
    }

    pub fn get_short(&self) -> Option<String> {
        let mut s = String::from('-');
        s.push(self.short?);
        Some(s)
    }

    pub fn short(mut self, s: char) -> Self {
        self.short = Some(s);
        self
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "--{}", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Optional {
    name: Flag,
    value: Positional,
}

impl Optional {
    pub fn new(s: &str) -> Self {
        Optional { 
            name: Flag::new(s),
            value: Positional::new(s),
        }
    }
    
    pub fn get_flag(&self) -> &Flag {
        &self.name
    }

    pub fn short(mut self, s: char) -> Self {
        self.name = self.name.short(s);
        self
    }

    pub fn value(mut self, s: &str) -> Self {
        self.value.name = s.to_string();
        self
    }
}

impl Display for Optional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{} {}", self.name, self.value)
    }
}

#[derive(Debug, PartialEq)]
pub enum Arg {
    Positional(Positional),
    Flag(Flag),
    Optional(Optional),
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::Flag(a) => write!(f, "{}", a),
            Self::Positional(a) => write!(f, "{}", a),
            Self::Optional(a) => write!(f, "{}", a),
        }
    }
}