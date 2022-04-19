//! File     : cfgfile.rs
//! Abstract :
//!     A `cfgfile` is the main file format used to store data for Orbit. It
//!     resembles a ini-like syntax and structure composed of "tables" 
//!     (sections) and "fields" (key-value pairs).
use std::collections::HashMap;
use crate::core::cfg::field;
use std::str::FromStr;

type Line = usize;
type Col = usize;
#[derive(Debug, PartialEq, Clone)]
struct Pos(Line, Col);

#[derive(Debug, PartialEq)]
enum TokenType {
    COMMENT(String),    // ; or #
    ASSIGNMENT,         // =
    EOL,                // \n
    RBRACKET,           // [
    LBRACKET,           // ]
    LITERAL(String),
    QUOTE(char),        // ' or "
    ENDQUOTE(char),     // ' or "
    EOF,
}

impl TokenType {
    fn from_char(c: char) -> Self {
        match c {
            '\'' | '\"' => Self::QUOTE(c),
            '=' => Self::ASSIGNMENT,
            ']' => Self::RBRACKET,
            '[' => Self::LBRACKET,
            _ => panic!("invalid operator!")
        }
    }

    fn as_operator(&self) -> Result<char, ()> {
        Ok(match self {
            Self::ASSIGNMENT => '=',
            Self::RBRACKET => ']',
            Self::LBRACKET => '[',
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
            Self::LITERAL(l) => write!(f, "{}", l),
            Self::QUOTE(q) => write!(f, "{}", q),
            Self::ENDQUOTE(q) => write!(f, "{}", q),
            Self::EOF => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Symbol {
    location: Pos,
    token: TokenType,
}

impl Symbol {
    pub fn new(pos: Pos, token: TokenType) -> Self {
        Self {
            location: pos,
            token: token,
        }
    }

    pub fn get_token(&self) -> &TokenType {
        &self.token
    }

    pub fn take_str(self) -> String {
        match self.token {
            TokenType::LITERAL(s) => s,
            TokenType::COMMENT(s) => s,
            _ => panic!("this token does not own a String")
        }
    }
}

enum CfgState {
    COMMENT,
    QUOTE(char),
    NORMAL,
}

struct CfgLanguage {
    map: HashMap::<field::Identifier, field::Value>,
}

impl CfgLanguage {
    fn new() -> Self {
        CfgLanguage { 
            map: HashMap::new(),
            // for saving, also store a list of the explicit table names mapped to list of sub key names
            // key is explicit table id, value a list of partial key ids
        }
    }

    /// Access the value behind a key.
    pub fn get(&self, s: &str) -> Option<&field::Value> {
        self.map.get(&field::Identifier::from_str(s).expect("invalid key format"))
    }

    /// Given a stream of tokens, build up hashmap according to the grammar.
    fn parse(tokens: Vec::<Symbol>) -> Result<HashMap::<field::Identifier, field::Value>, CfgError> {
        // track the current table name
        let mut table: Option<field::Identifier> = None;

        let mut map = HashMap::new();
        let mut t_stream = tokens.into_iter().peekable();
        while let Some(t) = t_stream.peek() {
            match t.get_token() {
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
                    panic!("unexpected token {:?}", t)
                }
            };
        }
        Ok(map)
    }

    /// FIELD ::= IDENTIFIER __=__ (BASIC_VALUE | LITERAL_VALUE)
    fn build_field(ts: &mut impl Iterator<Item=Symbol>) -> Result<(field::Identifier, field::Value), CfgError> {
        let mut ts = ts.peekable();
        // verify identifier and do something with it
        let key = CfgLanguage::verify_identifier(ts.next().unwrap())?;
        // verify that the next token is a '='
        CfgLanguage::accept_op(ts.next().unwrap(), '=')?;
        // accept accept basic literal || quoted literal || EOL/EOF
        let value = match ts.peek().unwrap().get_token() {
            TokenType::LITERAL(_) => {
                field::Value::from_move(ts.next().unwrap().take_str())
            }
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
            TokenType::EOL | TokenType::EOF => {
                field::Value::from_str("").unwrap()
            }
            _ => panic!("invalid token when parsing literal {:?}", ts.next().unwrap())
        };
        // accept EOL or EOF
        match ts.next().unwrap().get_token() {
            TokenType::EOF | TokenType::EOL => Ok((key, value)),
            _ => Err(CfgError::MissingEOL),
        }
    }

    /// Consumes an operator if it is matching `c` or reports an error.
    fn accept_op(t: Symbol, c: char) -> Result<(), CfgError> {
        if let Ok(v) = t.get_token().as_operator() {
            if v == c {
                Ok(())
            } else {
                panic!("unexpected operator {:?}", t)
            }
        } else {
            panic!("unexpected token {:?}", t)
        }
    }

    /// Verify the identifier is valid. It may contain only ascii letters and numbers, dashes,
    /// and dots.
    fn verify_identifier(t: Symbol) -> Result<field::Identifier, CfgError> {
        match t.get_token() {
            TokenType::LITERAL(_) => {
                match field::Identifier::from_move(t.take_str()) {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CfgError::InvalidIdentifier(e)),
                }
            },
            TokenType::EOF => {
                panic!("missing identifier")
            }
            _ => {
                panic!("unexpected token {:?}", t)
            }
        }
    }

    /// TABLE ::= __\[__ IDENTIFIER __\]__
    fn build_table(ts: &mut impl Iterator<Item=Symbol>) -> Result<field::Identifier, CfgError> {
        // accept [ ...guaranteed to be LBRACKET
        CfgLanguage::accept_op(ts.next().unwrap(), '[')?;
        // verify identifier
        let table = CfgLanguage::verify_identifier(ts.next().unwrap())?;
        // accept ]
        CfgLanguage::accept_op(ts.next().unwrap(), ']')?;
        // accept EOL or EOF
        match ts.next().unwrap().get_token() {
            TokenType::EOF | TokenType::EOL => Ok(table),
            _ => Err(CfgError::MissingEOL),
        }
    }
    
    /// Given some text `s`, tokenize it according the cfg language.
    fn tokenize(s: &str) -> Vec::<Symbol> {
        let mut symbols = Vec::new();
        let mut cur_pos = Pos(1, 0);
        let mut buf: String = String::new();
        let mut buf_pos: Pos = cur_pos.clone();
        let mut state = CfgState::NORMAL;

        let complete_literal = |v: &mut Vec::<Symbol>, p: &Pos, b: &str| {
            if b.is_empty() == false {
                v.push(Symbol::new(p.clone(), TokenType::LITERAL(b.to_owned())));
            }
        };

        let mut chars = s.chars().peekable();
        // main state machine logic for handling each character
        while let Some(c) = chars.next() {
            cur_pos.1 += 1;
            match state {
                CfgState::COMMENT => {
                    match c {
                        '\n' => {
                            symbols.push(Symbol::new(buf_pos.clone(), TokenType::COMMENT(buf.to_string())));
                            buf.clear();
                            symbols.push(Symbol::new(cur_pos.clone(), TokenType::EOL));
                            state = CfgState::NORMAL;
                            cur_pos.0 += 1;
                            cur_pos.1 = 0;
                        }
                        _ => {
                            buf.push(c);
                        }
                    }
                }
                CfgState::NORMAL => {
                    match c {
                        ';' | '#' => {
                            complete_literal(&mut symbols, &mut buf_pos, buf.trim());
                            buf.clear();
                            state = CfgState::COMMENT;
                            buf.push(c);
                            // mark where the comment begins
                            buf_pos = cur_pos.clone();
                        }
                        ']' | '[' | '=' | '\"' | '\'' => {
                            complete_literal(&mut symbols, &mut buf_pos, buf.trim());
                            buf.clear();
                            symbols.push(Symbol::new(cur_pos.clone(), TokenType::from_char(c)));
                            if c == '\"' || c == '\'' { 
                                state = CfgState::QUOTE(c);
                                // mark where the quote begins
                                buf_pos = cur_pos.clone();
                                buf_pos.1 += 1;
                            };
                        }
                        '\n' => {
                            buf = buf.trim().to_string();
                            complete_literal(&mut symbols, &mut buf_pos, buf.trim());
                            buf.clear();
                            symbols.push(Symbol::new(cur_pos.clone(), TokenType::EOL));
                            cur_pos.0 += 1;
                            cur_pos.1 = 0;

                        }
                        _ => {
                            if (c.is_whitespace() == false) || (buf.is_empty() == false) {
                                // mark the beginning location for this literal
                                if buf.is_empty() == true {
                                    buf_pos = cur_pos.clone();
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
                            cur_pos.1 += 1;
                        // finish the quoted literal
                        } else {
                            complete_literal(&mut symbols, &mut buf_pos, &buf);
                            buf.clear();
                            symbols.push(Symbol::new(cur_pos.clone(), TokenType::ENDQUOTE(q)));
                            state = CfgState::NORMAL;
                        }
                    } else {
                        buf.push(c);
                        if c == '\n' {
                            cur_pos.0 += 1;
                            cur_pos.1 = 0;
                        }
                    }
                }
            }
        }
        // final check to ensure emptying the buffer
        match state {
            CfgState::COMMENT => {
                symbols.push(Symbol::new(buf_pos, TokenType::COMMENT(buf)));
            },
            CfgState::NORMAL => {
                buf = buf.trim().to_string();
                complete_literal(&mut symbols, &mut buf_pos, &mut buf);
            }
            CfgState::QUOTE(_) => {
                complete_literal(&mut symbols, &mut buf_pos, &mut buf);
            },
        }
        cur_pos.1 += 1;
        symbols.push(Symbol::new(cur_pos, TokenType::EOF));
        symbols    
    }
}

#[derive(Debug, PartialEq)]
enum CfgError {
    InvalidIdentifier(field::IdentifierError),
    MissingOperator(char),
    MissingEOL,
    // ExpectedOperator(Token, char),
    /// (position, expected, got)
    InvalidOperator(Pos, char, char),
    // ExpectedEOL(Token),
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_key() {
        let v = vec![
            Symbol::new(Pos(1, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Pos(1, 2), TokenType::ASSIGNMENT),
            Symbol::new(Pos(1, 3), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Pos(1, 4), TokenType::EOL),
        ];
        assert_eq!(CfgLanguage::build_field(&mut v.into_iter()).unwrap(), 
            (field::Identifier::from_str("key1").unwrap(), field::Value::from_str("value").unwrap()));
            
        // only one key can be defined on a line (missing eol)
        let v = vec![
            Symbol::new(Pos(1, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Pos(1, 2), TokenType::ASSIGNMENT),
            Symbol::new(Pos(1, 3), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Pos(2, 1), TokenType::LITERAL("key2".to_owned())),
            Symbol::new(Pos(2, 2), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 3), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Pos(2, 4), TokenType::EOL),
        ];
        assert!(CfgLanguage::build_field(&mut v.into_iter()).is_err());
    }

    #[test]
    fn parse_table() {
        let v = vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 7), TokenType::RBRACKET),
            Symbol::new(Pos(1, 8), TokenType::EOL),
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), field::Identifier::from_str("table").unwrap());

        let v = vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("CORE".to_owned())),
            Symbol::new(Pos(1, 6), TokenType::RBRACKET),
            Symbol::new(Pos(1, 7), TokenType::EOF),
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), field::Identifier::from_str("CORE").unwrap());

        // only one table can be defined on a line
        let v = vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("CORE".to_owned())),
            Symbol::new(Pos(1, 3), TokenType::RBRACKET),
            Symbol::new(Pos(1, 4), TokenType::LBRACKET),
            Symbol::new(Pos(1, 5), TokenType::LITERAL("SUPERCORE".to_owned())),
            Symbol::new(Pos(1, 6), TokenType::RBRACKET),
            Symbol::new(Pos(1, 7), TokenType::EOF),
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
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 7), TokenType::RBRACKET),
            Symbol::new(Pos(1, 8), TokenType::EOL),
            Symbol::new(Pos(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 7), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Pos(2, 12), TokenType::EOL),
            Symbol::new(Pos(3, 1), TokenType::EOF),
        ]);

        let s = "\
[table]
key = place the value here
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 7), TokenType::RBRACKET),
            Symbol::new(Pos(1, 8), TokenType::EOL),
            Symbol::new(Pos(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 7), TokenType::LITERAL("place the value here".to_owned())),
            Symbol::new(Pos(2, 27), TokenType::EOL),
            Symbol::new(Pos(3, 1), TokenType::EOF),
        ]);

        let s = "\
[table]
key = \"value\"
jot = 'notes'
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 7), TokenType::RBRACKET),
            Symbol::new(Pos(1, 8), TokenType::EOL),
            Symbol::new(Pos(2, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(2, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 7), TokenType::QUOTE('"')),
            Symbol::new(Pos(2, 8), TokenType::LITERAL("value".to_owned())),
            Symbol::new(Pos(2, 13), TokenType::ENDQUOTE('"')),
            Symbol::new(Pos(2, 14), TokenType::EOL),
            Symbol::new(Pos(3, 1), TokenType::LITERAL("jot".to_owned())),
            Symbol::new(Pos(3, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(3, 7), TokenType::QUOTE('\'')),
            Symbol::new(Pos(3, 8), TokenType::LITERAL("notes".to_owned())),
            Symbol::new(Pos(3, 13), TokenType::ENDQUOTE('\'')),
            Symbol::new(Pos(3, 14), TokenType::EOL),
            Symbol::new(Pos(4, 1), TokenType::EOF),
        ]);
    }

    #[test]
    fn quoted_value() {
        // using comment operator in value
        let s = "key =\"value; more value! \"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Pos(1, 7), TokenType::LITERAL("value; more value! ".to_owned())),
            Symbol::new(Pos(1, 26), TokenType::ENDQUOTE('"')),
            Symbol::new(Pos(1, 27), TokenType::EOF),
        ]);

        // missing trailing quote
        let s = "\
key =\"[value;]=";
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Pos(1, 7), TokenType::LITERAL("[value;]=".to_owned())),
            Symbol::new(Pos(1, 16), TokenType::EOF),
        ]);

        // inserting newline and escaping quotes
        let s = "key =\"'orbit' is an HDL \n\"\"package manager\"\"\"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LITERAL("key".to_owned())),
            Symbol::new(Pos(1, 5), TokenType::ASSIGNMENT),
            Symbol::new(Pos(1, 6), TokenType::QUOTE('"')),
            Symbol::new(Pos(1, 7), TokenType::LITERAL("'orbit' is an HDL \n\"package manager\"".to_string())),
            Symbol::new(Pos(2, 20), TokenType::ENDQUOTE('"')),
            Symbol::new(Pos(2, 21), TokenType::EOF),
        ]);
    }

    #[test]
    fn spacing_and_eof() {
        let s = "\
[table]
key1 = value1


key2 = value2";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::LBRACKET),
            Symbol::new(Pos(1, 2), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 7), TokenType::RBRACKET),
            Symbol::new(Pos(1, 8), TokenType::EOL),
            Symbol::new(Pos(2, 1), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Pos(2, 6), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 8), TokenType::LITERAL("value1".to_owned())),
            Symbol::new(Pos(2, 14), TokenType::EOL),
            Symbol::new(Pos(3, 1), TokenType::EOL),
            Symbol::new(Pos(4, 1), TokenType::EOL),
            Symbol::new(Pos(5, 1), TokenType::LITERAL("key2".to_owned())),
            Symbol::new(Pos(5, 6), TokenType::ASSIGNMENT),
            Symbol::new(Pos(5, 8), TokenType::LITERAL("value2".to_owned())),
            Symbol::new(Pos(5, 14), TokenType::EOF),
        ]);

        let s = "    [table]
  key1=  value1 ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 5), TokenType::LBRACKET),
            Symbol::new(Pos(1, 6), TokenType::LITERAL("table".to_owned())),
            Symbol::new(Pos(1, 11), TokenType::RBRACKET),
            Symbol::new(Pos(1, 12), TokenType::EOL),
            Symbol::new(Pos(2, 3), TokenType::LITERAL("key1".to_owned())),
            Symbol::new(Pos(2, 7), TokenType::ASSIGNMENT),
            Symbol::new(Pos(2, 10), TokenType::LITERAL("value1".to_owned())),
            Symbol::new(Pos(2, 17), TokenType::EOF),
        ]);
    }

    #[test]
    fn comments() {
        let s = "\
; For more information visit orbit's website.
[core]
user = CHASE # your name or \"alias\"! ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Symbol::new(Pos(1, 1), TokenType::COMMENT("; For more information visit orbit's website.".to_string())),
            Symbol::new(Pos(1, 46), TokenType::EOL),
            Symbol::new(Pos(2, 1), TokenType::LBRACKET),
            Symbol::new(Pos(2, 2), TokenType::LITERAL("core".to_owned())),
            Symbol::new(Pos(2, 6), TokenType::RBRACKET),
            Symbol::new(Pos(2, 7), TokenType::EOL),
            Symbol::new(Pos(3, 1), TokenType::LITERAL("user".to_owned())),
            Symbol::new(Pos(3, 6), TokenType::ASSIGNMENT),
            Symbol::new(Pos(3, 8), TokenType::LITERAL("CHASE".to_owned())),
            Symbol::new(Pos(3, 14), TokenType::COMMENT("# your name or \"alias\"! ".to_string())),
            Symbol::new(Pos(3, 38), TokenType::EOF),
        ]);
    }

    #[test]
    fn parse() {
        let s = "\
; orbit configuration file

include.path = profile/eastwind-trading/config.ini,

[core]
path = /users/chase/hdl
user = 'Chase Ruskin '

[env]
course=EEL4712C: Digital Design 

[table]
key     = 
";
        let tokens = CfgLanguage::tokenize(s);
        let map = CfgLanguage::parse(tokens).unwrap();
        let config = CfgLanguage {
            map: map,
        };

        assert_eq!(config.get("core.path"), Some(&field::Value::from_str("/users/chase/hdl").unwrap()));
        assert_eq!(config.get("core.user"), Some(&field::Value::from_str("Chase Ruskin ").unwrap()));
        assert_eq!(config.get("table.key"), Some(&field::Value::from_str("").unwrap()));
        assert_eq!(config.get("plugin.ghdl.execute"), None);
    }
}