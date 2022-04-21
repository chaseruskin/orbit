//! File     : cfgfile.rs
//! Abstract :
//!     A `cfgfile` is the main file format used to store data for Orbit. It
//!     resembles a ini-like syntax and structure composed of "tables" 
//!     (sections) and "fields" (key-value pairs).
use std::collections::HashMap;
use crate::util::cfg::field;
use std::str::FromStr;

type Line = usize;
type Col = usize;
#[derive(Debug, PartialEq, Clone)]
pub struct Location(Line, Col);

impl Location {
    /// Starts a new position at line 1, column 0.
    fn new() -> Self {
        Location(1, 0)
    }

    /// Line += 1 and resets col to 0.
    fn increment_line(&mut self) {
        self.0 += 1;
        self.1 = 0;
    }

    /// Col += 1 and does not modify line.
    fn increment_col(&mut self) {
        self.1 += 1;
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    COMMENT(String),    // ; or #
    ASSIGNMENT,         // =
    EOL,                // \n
    RBRACKET,           // [
    LBRACKET,           // ]
    LITERAL(String),
    QUOTE(char),        // ' or "
    ENDQUOTE(char),     // ' or "
    COMMA,
    EOF,
}

impl TokenType {
    fn from_char(c: char) -> Self {
        match c {
            '\'' | '\"' => Self::QUOTE(c),
            '=' => Self::ASSIGNMENT,
            ']' => Self::RBRACKET,
            '[' => Self::LBRACKET,
            ',' => Self::COMMA,
            _ => panic!("invalid operator character \"{}\"", c)
        }
    }

    fn as_operator(&self) -> Result<char, ()> {
        Ok(match self {
            Self::ASSIGNMENT => '=',
            Self::RBRACKET => ']',
            Self::LBRACKET => '[',
            Self::COMMA => ',',
            Self::QUOTE(q) => *q,
            Self::ENDQUOTE(q) => *q,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::COMMENT(c) => write!(f, "{}", c),
            Self::ASSIGNMENT => write!(f, "="),
            Self::EOL => write!(f, "newline"),
            Self::RBRACKET => write!(f, "]"),
            Self::LBRACKET => write!(f, "["),
            Self::COMMA => write!(f, ","),
            Self::LITERAL(l) => write!(f, "{}", l),
            Self::QUOTE(q) => write!(f, "{}", q),
            Self::ENDQUOTE(q) => write!(f, "{}", q),
            Self::EOF => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Symbol {
    location: Location,
    token: TokenType,
}

impl Symbol {
    pub fn new(location: Location, token: TokenType) -> Self {
        Self {
            location: location,
            token: token,
        }
    }

    pub fn get_token(&self) -> &TokenType {
        &self.token
    }

    pub fn get_location(&self) -> &Location {
        &self.location
    }

    pub fn take_str(self) -> String {
        match self.token {
            TokenType::LITERAL(s) => s,
            TokenType::COMMENT(s) => s,
            _ => panic!("this token does not own a String {}", self.token)
        }
    }
}

enum CfgState {
    COMMENT,
    QUOTE(char),
    NORMAL,
}

pub struct CfgLanguage {
    map: HashMap::<field::Identifier, field::Value>,
    // for saving, also store a list of the explicit table names mapped to list of sub key names
    // key is explicit table id, value a list of partial key ids
}

// api impl block
impl CfgLanguage {
    pub fn new() -> Self {
        CfgLanguage { 
            map: HashMap::new(), 
        }
    }

    /// Accesses the value behind a key.
    /// 
    /// Returns `None` if the key does not exist.
    pub fn get(&self, s: &str) -> Option<&field::Value> {
        self.map.get(&field::Identifier::from_str(s).expect("invalid key format"))
    }

    pub fn load_from_file(f: &std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read_to_string(f)?;
        let cfg = CfgLanguage::from_str(&data)?;
        Ok(cfg)
    }
}

// lexer and parser impl block
impl CfgLanguage {
    /// Given a stream of tokens, build up hashmap according to the grammar.
    fn parse(tokens: Vec::<Symbol>) -> Result<HashMap::<field::Identifier, field::Value>, CfgError> {
        // track the current table name
        let mut table: Option<field::Identifier> = None;
        let mut map = HashMap::new();
        let mut t_stream = tokens.into_iter().peekable();
        while let Some(sym) = t_stream.peek() {
            match sym.get_token() {
                // define a table
                TokenType::LBRACKET => {
                    table = Some(CfgLanguage::build_table(&mut t_stream)?);
                    // :todo: add this explicit table name (preserve case sense) to a different map for later saving
                }
                // create a key
                TokenType::LITERAL(_) => {
                    let (key, val) = CfgLanguage::build_field(&mut t_stream)?;
                    // add data to the hashmap (case-insensitive keys)
                    if let Some(section) = &table {
                        // prefix the base to the key name
                        map.insert(key.prepend(section), val);
                    } else {
                        map.insert(key, val);
                    }
                }
                // move along in the stream
                TokenType::COMMENT(_) | TokenType::EOL | TokenType::EOF => {
                    t_stream.next();
                }
                _ => {
                    return Err(CfgError::UnexpectedToken(t_stream.next().unwrap()))
                }
            };
        }
        Ok(map)
    }

    /// TABLE ::= __\[__ IDENTIFIER __\]__ EOL
    fn build_table(ts: &mut impl Iterator<Item=Symbol>) -> Result<field::Identifier, CfgError> {
        // accept [ ...guaranteed to be LBRACKET
        CfgLanguage::accept_op(ts.next().unwrap(), '[')?;
        // verify identifier
        let table = CfgLanguage::verify_identifier(ts.next().unwrap())?;
        // accept ]
        CfgLanguage::accept_op(ts.next().unwrap(), ']')?;
        // accept EOL or EOF
        let sym = ts.next().unwrap();
        match sym.get_token() {
            TokenType::EOF | TokenType::EOL | TokenType::COMMENT(_) => Ok(table),
            _ => Err(CfgError::MissingEOL(sym.get_location().clone())),
        }
    }

    /// LITERAL ::= __"__ IDENTIFIER __"__
    fn build_literal(ts: &mut impl Iterator<Item=Symbol>) -> Result<field::Value, CfgError> {
        let mut ts = ts.peekable();
        Ok(match ts.peek().unwrap().get_token() {
            TokenType::QUOTE(_) => {
                // check what quote was used
                let q = ts.peek().unwrap().get_token().as_operator().unwrap();
                // consume the quote
                CfgLanguage::accept_op(ts.next().unwrap(), q)?;
                // capture the literal
                let v = field::Value::from_move(ts.next().unwrap().take_str());
                // consume closing quote
                CfgLanguage::accept_op(ts.next().unwrap(), q)?;
                v
            }
            TokenType::EOL | TokenType::EOF | TokenType::COMMENT(_) => {
                field::Value::from_str("").unwrap()
            }
            _ => {
                // panic!("invalid token when parsing literal {:?}", ts.next().unwrap())
                return Err(CfgError::UnexpectedToken(ts.next().unwrap()))
            }
        })
    }

    /// ARRAY ::= __\[__ LITERAL (__,__ ...) __\]__
    fn build_array(ts: &mut impl Iterator<Item=Symbol>) -> Result<field::Value, CfgError> {
        // accept left bracket
        CfgLanguage::accept_op(ts.next().unwrap(), '[')?;
        // enter variable length decoding process
        let mut ts = ts.peekable();

        let mut root_value: Option<field::Value> = None;
        let mut accepted_comma = true;
        loop {
            match ts.peek().unwrap().get_token() {
                TokenType::RBRACKET => break,
                TokenType::COMMENT(_) | TokenType::EOL | TokenType::EOF => {
                    ts.next().unwrap();
                },
                _ => {
                    // accept a literal (only if accepted a comma previously)
                    if accepted_comma == true {
                        let v = CfgLanguage::build_literal(&mut ts)?;
                        root_value = match root_value.take() {
                            Some(mut rv) => {
                                rv.push_value(v);
                                Some(rv)
                            }
                            None => Some(v),
                        };
                        accepted_comma = false;
                    } else {
                       return Err(CfgError::ExpectingComma(ts.next().unwrap()))
                    }
                    // accept a comma (if present)
                    if ts.peek().unwrap().get_token() == &TokenType::COMMA {
                        CfgLanguage::accept_op(ts.next().unwrap(), ',')?;
                        accepted_comma = true;
                    };
                }
            }
        }
        CfgLanguage::accept_op(ts.next().unwrap(), ']')?;
        Ok(root_value.unwrap_or(field::Value::new("")))
    }

    /// FIELD ::= IDENTIFIER __=__ (LITERAL | ARRAY) EOL
    fn build_field(ts: &mut impl Iterator<Item=Symbol>) -> Result<(field::Identifier, field::Value), CfgError> {
        let mut ts = ts.peekable();
        // verify identifier and do something with it
        let key = CfgLanguage::verify_identifier(ts.next().unwrap())?;
        // verify that the next token is a '='
        CfgLanguage::accept_op(ts.next().unwrap(), '=')?;
        // accept accept quote || quoted literal || EOL/EOF
        let value = match ts.peek().unwrap().get_token() {
            TokenType::LBRACKET => {
                CfgLanguage::build_array(&mut ts)?
            }
            TokenType::EOF => {
                field::Value::new("")
            }
            _ => {
                CfgLanguage::build_literal(&mut ts)?
            }
        };
        // accept EOL or EOF
        let sym = ts.next().unwrap();
        match sym.get_token() {
            TokenType::EOF | TokenType::EOL | TokenType::COMMENT(_) => Ok((key, value)),
            _ => Err(CfgError::MissingEOL(sym.get_location().clone())),
        }
    }

    /// Consumes an operator if it is matching `c` or reports an error.
    fn accept_op(sym: Symbol, c: char) -> Result<(), CfgError> {
        if let Ok(v) = sym.get_token().as_operator() {
            if v == c {
                Ok(())
            } else {
                Err(CfgError::UnexpectedToken(sym))
            }
        } else {
            Err(CfgError::UnexpectedToken(sym))
        }
    }

    /// Verify the identifier is valid. It may contain only ascii letters and numbers, dashes,
    /// and dots.
    fn verify_identifier(sym: Symbol) -> Result<field::Identifier, CfgError> {
        match sym.get_token() {
            TokenType::LITERAL(s) => {
                match field::Identifier::from_str(s.as_ref()) {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CfgError::InvalidIdentifier(sym, e)),
                }
            },
            TokenType::EOF => {
                panic!("missing identifier")
            }
            _ => {
                Err(CfgError::UnexpectedToken(sym))
            }
        }
    }
    
    /// Given some text `s`, tokenize it according the cfg language.
    fn tokenize(s: &str) -> Vec::<Symbol> {
        let mut symbols = Vec::new();
        // tracks the tokenizer's current position
        let mut cur_loc = Location::new();
        // tracks the buffer's intitial position
        let mut buf_loc = Location::new();
        let mut buf: String = String::new();
        let mut state = CfgState::NORMAL;

        let complete_literal = |v: &mut Vec::<Symbol>, p: &Location, b: &str| {
            if b.is_empty() == false {
                v.push(Symbol::new(p.clone(), TokenType::LITERAL(b.to_owned())));
            }
        };

        let mut chars = s.chars().peekable();
        // main state machine logic for handling each character
        while let Some(c) = chars.next() {
            cur_loc.increment_col();
            match state {
                CfgState::COMMENT => {
                    match c {
                        '\n' => {
                            symbols.push(Symbol::new(buf_loc.clone(), TokenType::COMMENT(buf.to_string())));
                            buf.clear();
                            symbols.push(Symbol::new(cur_loc.clone(), TokenType::EOL));
                            state = CfgState::NORMAL;
                            cur_loc.increment_line();
                        }
                        _ => {
                            buf.push(c);
                        }
                    }
                }
                CfgState::NORMAL => {
                    match c {
                        ';' | '#' => {
                            complete_literal(&mut symbols, &mut buf_loc, buf.trim());
                            buf.clear();
                            state = CfgState::COMMENT;
                            buf.push(c);
                            // mark where the comment begins
                            buf_loc = cur_loc.clone();
                        }
                        ']' | '[' | '=' | '\"' | '\'' | ',' => {
                            complete_literal(&mut symbols, &mut buf_loc, buf.trim());
                            buf.clear();
                            symbols.push(Symbol::new(cur_loc.clone(), TokenType::from_char(c)));
                            if c == '\"' || c == '\'' { 
                                state = CfgState::QUOTE(c);
                                // mark where the quote begins
                                buf_loc = cur_loc.clone();
                                buf_loc.increment_col();
                            };
                        }
                        '\n' => {
                            buf = buf.trim().to_string();
                            complete_literal(&mut symbols, &mut buf_loc, buf.trim());
                            buf.clear();
                            symbols.push(Symbol::new(cur_loc.clone(), TokenType::EOL));
                            cur_loc.increment_line();
                        }
                        _ => {
                            if (c.is_whitespace() == false) || (buf.is_empty() == false) {
                                // mark the beginning location for this literal
                                if buf.is_empty() == true {
                                    buf_loc = cur_loc.clone();
                                }
                                buf.push(c);
                            }
                        }
                    }
                }
                CfgState::QUOTE(q) => {
                    if c == q {
                        if chars.peek() == Some(&q) {
                            // discard the escape quote
                            buf.push(chars.next().unwrap());
                            cur_loc.increment_col();
                        // finish the quoted literal
                        } else {
                            symbols.push(Symbol::new(buf_loc.clone(), TokenType::LITERAL(buf.to_owned())));
                            buf.clear();
                            symbols.push(Symbol::new(cur_loc.clone(), TokenType::ENDQUOTE(q)));
                            state = CfgState::NORMAL;
                        }
                    } else {
                        buf.push(c);
                        if c == '\n' {
                            cur_loc.increment_line();
                        }
                    }
                }
            }
        }
        // final check to ensure emptying the buffer
        match state {
            CfgState::COMMENT => {
                symbols.push(Symbol::new(buf_loc, TokenType::COMMENT(buf)));
            },
            CfgState::NORMAL => {
                buf = buf.trim().to_string();
                complete_literal(&mut symbols, &mut buf_loc, &mut buf);
            }
            CfgState::QUOTE(_) => {
                complete_literal(&mut symbols, &mut buf_loc, &mut buf);
            },
        }
        cur_loc.increment_col();
        symbols.push(Symbol::new(cur_loc, TokenType::EOF));
        symbols    
    }
}

impl FromStr for CfgLanguage {
    type Err = CfgError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = CfgLanguage::tokenize(s);
        Ok(CfgLanguage {
            map: CfgLanguage::parse(tokens)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum CfgError {
    InvalidIdentifier(Symbol, field::IdentifierError),
    MissingEOL(Location),
    UnexpectedToken(Symbol),
    ExpectingComma(Symbol),
    // MissingOperator(char),

    // ExpectedOperator(Token, char),
    // (position, expected, got)
    // InvalidOperator(Location, char, char),
    // ExpectedEOL(Token),
}

impl std::error::Error for CfgError {}

impl std::fmt::Display for CfgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectingComma(sym) => write!(f, "{} expecting comma", sym.get_location()),
            Self::InvalidIdentifier(sym, e) => write!(f, "{} invalid identifier \'{}\' due to {}", sym.get_location(), sym.get_token(), e),
            Self::MissingEOL(l) => write!(f, "{} missing end of line", l),
            Self::UnexpectedToken(sym) => write!(f, "{} unexpected token \'{}\'", sym.get_location(), sym.get_token()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_key() {
        let v = vec![
            Symbol::new(Location(1, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Location(1, 2), TokenType::ASSIGNMENT),
            Symbol::new(Location(1, 3), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 4), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Location(1, 5), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 6), TokenType::EOL),
        ];
        assert_eq!(CfgLanguage::build_field(&mut v.into_iter()).unwrap(), 
            (field::Identifier::from_str("key1").unwrap(), field::Value::from_str("value").unwrap()));
            
        // only one key can be defined on a line (missing eol)
        let v = vec![
            Symbol::new(Location(1, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Location(1, 2), TokenType::ASSIGNMENT),
            Symbol::new(Location(1, 3), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 4), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Location(1, 5), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(2, 1), TokenType::LITERAL("key2".to_owned())),
            Symbol::new(Location(2, 2), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 3), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Location(2, 4), TokenType::EOL),
        ];
        assert!(CfgLanguage::build_field(&mut v.into_iter()).is_err());
    }

    #[test]
    fn parse_table() {
        let v = vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 7), TokenType::RBRACKET),
            Symbol::new(Location(1, 8), TokenType::EOL),
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), field::Identifier::from_str("table").unwrap());

        let v = vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("CORE".to_owned())),
            Symbol::new(Location(1, 6), TokenType::RBRACKET),
            Symbol::new(Location(1, 7), TokenType::EOF),
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), field::Identifier::from_str("CORE").unwrap());

        // only one table can be defined on a line
        let v = vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("CORE".to_owned())),
            Symbol::new(Location(1, 3), TokenType::RBRACKET),
            Symbol::new(Location(1, 4), TokenType::LBRACKET),
            Symbol::new(Location(1, 5), TokenType::LITERAL("SUPERCORE".to_owned())),
            Symbol::new(Location(1, 6), TokenType::RBRACKET),
            Symbol::new(Location(1, 7), TokenType::EOF),
        ];
        assert!(CfgLanguage::build_table(&mut v.into_iter()).is_err());
    }

    #[test]
    fn basic_lexer() {
        let s = "\
[table]
key = value
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 7), TokenType::RBRACKET),
            Symbol::new(Location(1, 8), TokenType::EOL),
            Symbol::new(Location(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 7), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Location(2, 12), TokenType::EOL),
            Symbol::new(Location(3, 1), TokenType::EOF),
        ]);

        let s = "\
[table]
key = place the value here
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 7), TokenType::RBRACKET),
            Symbol::new(Location(1, 8), TokenType::EOL),
            Symbol::new(Location(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 7), TokenType::LITERAL("place the value here".to_owned())),
            Symbol::new(Location(2, 27), TokenType::EOL),
            Symbol::new(Location(3, 1), TokenType::EOF),
        ]);

        let s = "\
[table]
key = \"value\"
jot = 'notes'
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 7), TokenType::RBRACKET),
            Symbol::new(Location(1, 8), TokenType::EOL),
            Symbol::new(Location(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 7), TokenType::QUOTE('"')),
            Symbol::new(Location(2, 8), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Location(2, 13), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(2, 14), TokenType::EOL),
            Symbol::new(Location(3, 1), TokenType::LITERAL("jot".to_owned())),
            Symbol::new(Location(3, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(3, 7), TokenType::QUOTE('\'')),
            Symbol::new(Location(3, 8), TokenType::LITERAL("notes".to_owned())),
            Symbol::new(Location(3, 13), TokenType::ENDQUOTE('\'')),
            Symbol::new(Location(3, 14), TokenType::EOL),
            Symbol::new(Location(4, 1), TokenType::EOF),
        ]);
    }

    #[test]
    fn empty_value() {
        let s = "\"\"";
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 2), TokenType::LITERAL("".to_owned())),
            Symbol::new(Location(1, 2), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 3), TokenType::EOF),
        ]);
    }

    #[test]
    fn quoted_value() {
        // using comment operator in value
        let s = "key =\"value; more value! \"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 7), TokenType::LITERAL("value; more value! ".to_owned())),
            Symbol::new(Location(1, 26), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 27), TokenType::EOF),
        ]);

        // missing trailing quote
        let s = "\
key =\"[value;]=";
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 7), TokenType::LITERAL("[value;]=".to_owned())),
            Symbol::new(Location(1, 16), TokenType::EOF),
        ]);

        // inserting newline and escaping quotes
        let s = "key =\"'orbit' is an HDL \n\"\"package manager\"\"\"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Location(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Location(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 7), TokenType::LITERAL("'orbit' is an HDL \n\"package manager\"".to_string())),
            Symbol::new(Location(2, 20), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(2, 21), TokenType::EOF),
        ]);
    }

    #[test]
    fn spacing_and_eof() {
        let s = "\
[table]
key1 = value1


key2 = value2";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 7), TokenType::RBRACKET),
            Symbol::new(Location(1, 8), TokenType::EOL),
            Symbol::new(Location(2, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Location(2, 6), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 8), TokenType::LITERAL("value1".to_owned())),
            Symbol::new(Location(2, 14), TokenType::EOL),
            Symbol::new(Location(3, 1), TokenType::EOL),
            Symbol::new(Location(4, 1), TokenType::EOL),
            Symbol::new(Location(5, 1), TokenType::LITERAL("key2".to_owned())),
            Symbol::new(Location(5, 6), TokenType::ASSIGNMENT),
            Symbol::new(Location(5, 8), TokenType::LITERAL("value2".to_owned())),
            Symbol::new(Location(5, 14), TokenType::EOF),
        ]);

        let s = "    [table]
  key1=  value1 ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 5), TokenType::LBRACKET),
            Symbol::new(Location(1, 6), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Location(1, 11), TokenType::RBRACKET),
            Symbol::new(Location(1, 12), TokenType::EOL),
            Symbol::new(Location(2, 3), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Location(2, 7), TokenType::ASSIGNMENT),
            Symbol::new(Location(2, 10), TokenType::LITERAL("value1".to_owned())),
            Symbol::new(Location(2, 17), TokenType::EOF),
        ]);
    }

    #[test]
    fn comma() {
        let s = "[\"a\",\"b\",\"c\"]";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::LBRACKET),
            Symbol::new(Location(1, 2), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 3), TokenType::LITERAL("a".to_owned())),
            Symbol::new(Location(1, 4), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 5), TokenType::COMMA),
            Symbol::new(Location(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 7), TokenType::LITERAL("b".to_owned())),
            Symbol::new(Location(1, 8), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 9), TokenType::COMMA),
            Symbol::new(Location(1, 10), TokenType::QUOTE('"')),
            Symbol::new(Location(1, 11), TokenType::LITERAL("c".to_owned())),
            Symbol::new(Location(1, 12), TokenType::ENDQUOTE('"')),
            Symbol::new(Location(1, 13), TokenType::RBRACKET),
            Symbol::new(Location(1, 14), TokenType::EOF),
        ]);   
    }

    #[test]
    fn comments() {
        let s = "\
; For more information visit orbit's website.
[core]
user = CHASE # your name or \"alias\"! ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Location(1, 1), TokenType::COMMENT("; For more information visit orbit's website.".to_string())),
            Symbol::new(Location(1, 46), TokenType::EOL),
            Symbol::new(Location(2, 1), TokenType::LBRACKET),
            Symbol::new(Location(2, 2), TokenType::LITERAL("core".to_owned())),
            Symbol::new(Location(2, 6), TokenType::RBRACKET),
            Symbol::new(Location(2, 7), TokenType::EOL),
            Symbol::new(Location(3, 1), TokenType::LITERAL("user".to_owned())),
            Symbol::new(Location(3, 6), TokenType::ASSIGNMENT),
            Symbol::new(Location(3, 8), TokenType::LITERAL("CHASE".to_owned())),
            Symbol::new(Location(3, 14), TokenType::COMMENT("# your name or \"alias\"! ".to_string())),
            Symbol::new(Location(3, 38), TokenType::EOF),
        ]);
    }

    #[test]
    fn from_str() {
        let s = "\
; orbit configuration file

include.path = 'profile/eastwind-trading/config.ini'

[core] ; comment
path = '/users/chase/hdl' ; comment #2
user = 'Chase Ruskin '

[env]
course= 'EEL4712C: Digital Design'
items = ['apples', 'bananas']

[table]
key     = ; comment #3
";
        let config = CfgLanguage::from_str(s).unwrap();

        assert_eq!(config.get("core.path"), Some(&field::Value::from_str("/users/chase/hdl").unwrap()));
        assert_eq!(config.get("CORE.USER"), Some(&field::Value::from_str("Chase Ruskin ").unwrap()));
        assert_eq!(config.get("table.key"), Some(&field::Value::from_str("").unwrap()));
        assert_eq!(config.get("plugin.ghdl.execute"), None);
        assert_eq!(config.get("table.key"), Some(&field::Value::from_str("").unwrap()));
        assert_eq!(config.get("env.items").unwrap().as_vec(), vec!["apples", "bananas"]);
    }
}