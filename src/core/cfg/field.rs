use std::str::FromStr;

#[derive(Debug, Hash)]
pub struct Identifier {
    id: String,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        // case insensitive
        self.id.to_lowercase() == other.id.to_lowercase()
    }
}

impl Eq for Identifier {}

impl Identifier {
    pub fn get_id(&self) -> &str {
        self.id.as_ref()
    }
    
    /// Joins the base path to the self's path to create a new `Identifier`.
    pub fn prepend(mut self, base: &Self) -> Identifier {
        self.id = [base.get_id(), self.get_id()].join(".");
        self
    }

    /// Ensures a given string follows the rules for denoting an identifier.
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

        // cannot have consecutive '.'
        if s.contains("..") == true {
            return Err(IdentifierError::EmptyTable)
        }
        
        match result {
            Some(c) => Err(IdentifierError::InvalidChar(c)),
            None =>  Ok(()),
        }
    }

    /// Takes the `String` and moves it into a new `Identifier` struct.
    pub fn from_move(s: String) -> Result<Self, IdentifierError> {
        Identifier::validate(&s)?;
        Ok(Identifier { id: s })
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> { 
        Identifier::validate(s)?;
        Ok(Identifier { id: s.to_owned() })
    }
}

#[derive(Debug, PartialEq)]
pub enum IdentifierError {
    Empty,
    InvalidChar(char),
    BadFirstChar(char),
    TrailingSep,
    EmptyTable,
}

impl std::error::Error for IdentifierError {}

impl std::fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::Empty => write!(f, "empty identifier"),
            Self::InvalidChar(c) => write!(f, "invalid character '{}' in identifier", c),
            Self::BadFirstChar(c) => write!(f, "invalid first character '{}'", c),
            Self::TrailingSep => write!(f, "invalid trailing separator '.'"),
            Self::EmptyTable => write!(f, "empty table identifier"),
        }
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
    /// Takes the `String` and moves it into a new `Value` struct.
    pub fn from_move(s: String) -> Self {
        Value { value: s }
    }

    /// Casts into a list split by 'sep'.
    pub fn as_vec(&self, sep: char) -> Vec<&str> {
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

#[derive(Debug, PartialEq)]
pub struct Field {
    key: Identifier,
    value: Value,
}

impl Field {
    pub fn get_key(&self) -> &Identifier {
        &self.key
    }

    pub fn get_value(&self) -> &Value {
        &self.value
    }

    pub fn take_split(self) -> (Identifier, Value) {
        (self.key, self.value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_from_str() {
        let id = Identifier::from_str("include.path");
        assert!(id.is_ok());

        let id = Identifier::from_str("plugin.ghdl.symbol.py-model");
        assert!(id.is_ok());

        let id = Identifier::from_str("name");
        assert!(id.is_ok());

        let id = Identifier::from_str("mykey?");
        assert!(id.is_err());

        // cannot end with a '.'
        let id = Identifier::from_str("core.");
        assert!(id.is_err());

        // cannot begin with a number
        let id = Identifier::from_str("9key");
        assert!(id.is_err());

        // cannot begin with a '.'
        let id = Identifier::from_str(".key");
        assert!(id.is_err());

        // cannot have a space
        let id = Identifier::from_str("core user");
        assert!(id.is_err());

        // cannot have a empty middle table
        let id = Identifier::from_str("core..user");
        assert!(id.is_err());
    }

    #[test]
    fn prepend() {
        let table = Identifier::from_str("core").unwrap();
        let key = Identifier::from_str("user").unwrap();

        assert_eq!(key.prepend(&table), Identifier {
            id: "core.user".to_owned(),
        });

        let key = Identifier::from_str("ghdl.symbol.py-model").unwrap();

        assert_eq!(key.prepend(&table), Identifier {
            id: "core.ghdl.symbol.py-model".to_owned(),
        });
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
    fn as_vec() {
        let v = Value::from_str("nor_gate,and_gate,mux_2x1").unwrap();
        assert_eq!(v.as_vec(','), ["nor_gate", "and_gate", "mux_2x1"]);

        let v = Value::from_str("").unwrap();
        assert_eq!(v.as_vec(','), Vec::<&str>::new());

        let v = Value::from_str("mux_2x1").unwrap();
        assert_eq!(v.as_vec(','), ["mux_2x1"]);

        let v = Value::from_str("profile/eel4712c/config.ini,").unwrap();
        assert_eq!(v.as_vec(','), ["profile/eel4712c/config.ini"]);

        let v = Value::from_str(",profile/eel4712c/config.ini").unwrap();
        assert_eq!(v.as_vec(','), ["", "profile/eel4712c/config.ini"]);
    }
}