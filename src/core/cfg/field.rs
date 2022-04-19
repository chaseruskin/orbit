use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Field {
    key: Identifier,
    value: Option<Value>,
}

#[derive(Debug, PartialEq)]
pub struct Identifier {
    id: String,
}

impl Identifier {
    pub fn get_id(&self) -> &str {
        self.id.as_ref()
    }

    fn validate(s: &str) -> Result<(), IdentifierError> {
        let mut chars = s.chars().peekable();
        match chars.next() {
            Some(c) => {
                // must begin with alphabetic character
                if c.is_alphabetic() == false {
                    return Err(IdentifierError::BadFirstChar(c))
                }
            }
            // must be not be empty
            None => return Err(IdentifierError::Empty)
        }
        // find first char in key not following spec
        let result = chars
            .find(|&c| {
                !c.is_alphanumeric() && !(c == '_') && !(c == '-') && !(c == '.')
            });

        // cannot end with a '.'
        if s.ends_with('.') == true {
            return Err(IdentifierError::TrailingSep)
        }
        
        match result {
            Some(c) => Err(IdentifierError::InvalidChar(c)),
            None =>  Ok(()),
        }
    }

    pub fn from_move(s: String) -> Result<Self, IdentifierError> {
        Identifier::validate(&s)?;
        Ok(Identifier { id: s })
    }
}

#[derive(Debug, PartialEq)]
pub enum IdentifierError {
    Empty,
    InvalidChar(char),
    BadFirstChar(char),
    TrailingSep,
}

impl std::fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::Empty => write!(f, "empty identifier"),
            Self::InvalidChar(c) => write!(f, "invalid character '{}' in identifier", c),
            Self::BadFirstChar(c) => write!(f, "invalid first character '{}'", c),
            Self::TrailingSep => write!(f, "invalid trailing separator '.'"),
        }
    }
}

impl std::error::Error for IdentifierError {}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> { 
        Identifier::validate(s)?;
        Ok(Identifier { id: s.to_owned() })
    }
}


#[derive(Debug, PartialEq)]
pub struct Value {
    value: String,
}

#[derive(Debug, PartialEq)]
pub enum ValueError {
    // todo
}

impl FromStr for Value {
    type Err = ValueError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> { 
        Ok(Value { value: s.to_owned() })
    }
}

impl Value {
    pub fn from_move(s: String) -> Self {
        Value { value: s }
    }

    pub fn as_list(&self, sep: char) -> Vec<&str> {
        self.value.split_terminator(sep).collect()
    }

    /// Returns true iff value is "YES", "ON", "1", "TRUE", or "ENABLE".
    pub fn as_bool(&self) -> bool {
        match self.value.to_lowercase().as_ref() {
            "yes" | "true" | "1" | "on" | "enable" => true,
            _ => false,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_from_str() {
        let key = Identifier::from_str("include.path");
        assert_eq!(key.is_ok(), true);

        let id = Identifier::from_str("plugin.ghdl.symbol.py-model");
        assert!(id.is_ok());

        let id = Identifier::from_str("mykey?");
        assert!(id.is_err());

        let id = Identifier::from_str("core.");
        assert!(id.is_err());

        let id = Identifier::from_str("9key");
        assert!(id.is_err());
    }

    #[test]
    fn new_value() {
        let v = Value::from_str("yes").unwrap(); 
        assert_eq!(v, Value { value: "yes".to_owned() });

        let v = String::from("hello world!");
        let v = Value::from_move(v);
        assert_eq!(v, Value { value: "hello world!".to_owned() });
    }

    #[test]
    fn as_bool() {
        let v = Value::from_str("yes").unwrap();
        assert_eq!(v.as_bool(), true);

        let v = Value::from_str("NO").unwrap();
        assert_eq!(v.as_bool(), false);

        let v = Value::from_str("ENABLE").unwrap();
        assert_eq!(v.as_bool(), true);

        let v = Value::from_str("12").unwrap();
        assert_eq!(v.as_bool(), false);
    }

    #[test]
    fn as_list() {
        let v = Value::from_str("nor_gate,and_gate,mux_2x1").unwrap();
        assert_eq!(v.as_list(','), ["nor_gate", "and_gate", "mux_2x1"]);

        let v = Value::from_str("").unwrap();
        assert_eq!(v.as_list(','), Vec::<&str>::new());

        let v = Value::from_str("mux_2x1").unwrap();
        assert_eq!(v.as_list(','), ["mux_2x1"]);

        let v = Value::from_str("profile/eel4712c/config.ini,").unwrap();
        assert_eq!(v.as_list(','), ["profile/eel4712c/config.ini"]);

        let v = Value::from_str(",profile/eel4712c/config.ini").unwrap();
        assert_eq!(v.as_list(','), ["", "profile/eel4712c/config.ini"]);
    }
}