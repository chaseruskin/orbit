//! File     : cfgfile.rs
//! Abstract :
//!     A `cfgfile` is the main file format used to store data for Orbit. It
//!     resembles a ini-like syntax and structure composed of "tables" 
//!     (sections) and "fields" (key-value pairs).
use std::collections::HashMap;

type Line = usize;
type Col = usize;
#[derive(Debug, PartialEq)]
struct Pos(Line, Col);

// strings " '
// operators [ ] =
// comment ; #
#[derive(Debug, PartialEq)]
enum Token {
    Comment(Pos, String),
    Operator(Pos, char),
    Identifier(Pos, String),
    EOL,
    EOF,
}

struct CfgLanguage {
    map: HashMap::<String, String>,
}

enum CfgState {
    COMMENT,
    QUOTE(char),
    NORMAL,
}

#[derive(Debug, PartialEq)]
enum CfgError {

}

impl CfgLanguage {
    fn new() -> Self {
        CfgLanguage { 
            map: HashMap::new(),
            // for saving, also store a list of the explicit table names mapped to list of sub key names
        }
    }

    /// Given a stream of tokens, build up hashmap according to the grammar.
    fn parse(tokens: Vec::<Token>) -> Result<HashMap::<String, String>, CfgError> {
        use Token::*;
        // track the current table name
        let mut table: Option<String> = None;

        let mut map = HashMap::new();
        let mut t_stream = tokens.into_iter().peekable();
        while let Some(t) = t_stream.peek() {
            match t {
                // define a table
                Operator(_, _) => {
                    table = Some(CfgLanguage::build_table(&mut t_stream)?);
                    // :todo: add this explicit table name (preserve case sense) to a different map for later saving
                }
                // create a key
                Identifier(_, _) => {
                    let (key, val) = CfgLanguage::build_field(&mut t_stream)?;
                    // :todo: add this explicit key name (preserve case sense) to a different map for later saving

                    // add data to the hashmap (case-insensitive keys)
                    if let Some(section) = &table {
                        map.insert([section.clone(), key].join("."), val);
                    } else {
                        map.insert(key, val);
                    }
                }
                // move along in the stream
                EOL | EOF | Comment(_, _) => {
                    t_stream.next();
                },
            };
        }
        Ok(map)
    }

    /// FIELD ::= KEY __=__ (BASIC_VALUE | LITERAL_VALUE)
    fn build_field(ts: &mut impl Iterator<Item=Token>) -> Result<(String, String), CfgError> {
        // verify identifier and do something with it
        // verify that the next token is a '='
        // accept value quoted literal || accept basic literal || EOL/EOF
        // return
        todo!()
    }

    fn accept_op(t: Option<Token>, c: char) -> Result<(), CfgError> {
        if let Some(tk) = t {
            match tk {
                Token::Operator(p, tc) => {
                    if tc == c {
                        Ok(())
                    } else {
                        panic!("bad operator")
                    }
                }
                _ => {
                    panic!("bad token")
                }
            }
        } else {
            panic!("missing token!")
        }
    }

    fn accept_terminator(t: Option<Token>) -> Result<(), CfgError> {
        if let Some(tk) = t {
            match tk {
                Token::EOL | Token::EOF => Ok(()),
                _ => panic!("unexpected token")
            }
        } else {
            panic!("missing token!")
        }
    }

    /// Verify the identifier is valid. It may contain only ascii letters and numbers, dashes,
    /// and dots.
    fn verify_identifier(t: Option<Token>) -> Result<String, CfgError> {
        if let Some(tk) = t {
            match tk {
                Token::Identifier(_, s) => {
                    // check that only ascii numbers and letters are allowed
                    for c in s.chars() {
                        if c.is_ascii_alphanumeric() == false && c != '.' && c != '-' {
                            panic!("invalid identifier")
                        }
                    }
                    Ok(s)
                },
                _ => panic!("unexpected token")
            }
        } else {
            panic!("missing token!")
        }
    }

    /// TABLE ::= __\[__ IDENTIFIER __\]__
    fn build_table(ts: &mut impl Iterator<Item=Token>) -> Result<String, CfgError> {
        // accept [
        CfgLanguage::accept_op(ts.next(), '[')?;
        // verify identifier and do something with it
        let result = CfgLanguage::verify_identifier(ts.next())?;
        // accept ]
        CfgLanguage::accept_op(ts.next(), ']')?;
        // accept EOL/EOF
        CfgLanguage::accept_terminator(ts.next())?;
        Ok(result)
    }
    
    /// Given some text `s`, tokenize it according the cfg language.
    fn tokenize(s: &str) -> Vec::<Token> {
        let mut tokens = Vec::new();
        let mut line: Line = 1;
        let mut col: Col = 0;
        let mut buf: String = String::new();
        let mut buf_pos: Pos = Pos(line, col);
        let mut state = CfgState::NORMAL;
        let mut chars = s.chars().peekable();

        let complete_token = |v: &mut Vec::<Token>, p: &Pos, b: &mut String| {
            if b.is_empty() == false {
                v.push(Token::Identifier(Pos(p.0, p.1), b.trim().to_string()));
                b.clear();
            }
        };

        while let Some(c) = chars.next() {
            col += 1;
            match state {
                CfgState::COMMENT => {
                    match c {
                        '\n' => {
                            tokens.push(Token::Comment(Pos(buf_pos.0, buf_pos.1), buf.to_string()));
                            buf.clear();
                            tokens.push(Token::EOL);
                            state = CfgState::NORMAL;
                            line += 1;
                            col = 0;
                        }
                        _ => {
                            buf.push(c);
                        }
                    }
                }
                CfgState::NORMAL => {
                    match c {
                        ';' | '#' => {
                            complete_token(&mut tokens, &mut buf_pos, &mut buf);
                            tokens.push(Token::Operator(Pos(line, col), c));
                            state = CfgState::COMMENT;
                            buf_pos = Pos(line, col+1);
                        }
                        ']' | '[' | '=' | '\"' | '\'' => {
                            complete_token(&mut tokens, &mut buf_pos, &mut buf);
                            tokens.push(Token::Operator(Pos(line, col), c));
                            if c == '\"' || c == '\'' { 
                                state = CfgState::QUOTE(c);
                                buf_pos = Pos(line, col+1);
                            };
                        }
                        '\n' => {
                            complete_token(&mut tokens, &mut buf_pos, &mut buf);
                            tokens.push(Token::EOL);
                            line += 1;
                            col = 0;
                        }
                        _ => {
                            if (c.is_whitespace() == false) || (buf.is_empty() == false) {
                                if buf.is_empty() == true {
                                    buf_pos = Pos(line, col);
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
                            col += 1;
                        } else {
                            complete_token(&mut tokens, &mut buf_pos, &mut buf);
                            tokens.push(Token::Operator(Pos(line, col), c));
                            state = CfgState::NORMAL;
                        }
                    } else {
                        buf.push(c);
                        if c == '\n' {
                            line += 1;
                            col = 0;
                        }
                    }
                }
            }
        }
        // final check to ensure emptying the buffer
        match state {
            CfgState::COMMENT => {
                tokens.push(Token::Comment(buf_pos, buf));
            },
            CfgState::NORMAL | CfgState::QUOTE(_) => {
                complete_token(&mut tokens, &mut buf_pos, &mut buf);
            },
        }
        tokens.push(Token::EOF);
        tokens
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    #[test]
    fn parse_table() {
        let v = vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), "table");

        let v = vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "CORE".to_string()),
            Operator(Pos(1, 6), ']'),
            EOF,
        ];
        assert_eq!(CfgLanguage::build_table(&mut v.into_iter()).unwrap(), "CORE");
    }

    #[test]
    fn basic_lexer() {
        let s = "\
[table]
key = value
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Identifier(Pos(2, 7), "value".to_string()),
            EOL,
            EOF,
        ]);

        let s = "\
[table]
key = place the value here
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Identifier(Pos(2, 7), "place the value here".to_string()),
            EOL,
            EOF,
        ]);

        let s = "\
[table]
key = \"value\"
jot = 'notes'
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Operator(Pos(2, 7), '"'),
            Identifier(Pos(2, 8), "value".to_string()),
            Operator(Pos(2, 13), '"'),
            EOL,
            Identifier(Pos(3, 1), "jot".to_string()),
            Operator(Pos(3, 5), '='),
            Operator(Pos(3, 7), '\''),
            Identifier(Pos(3, 8), "notes".to_string()),
            Operator(Pos(3, 13), '\''),
            EOL,
            EOF,
        ]);
    }

    #[test]
    fn quoted_value() {
        // using comment operator in value
        let s = "key =\"value; more value!\"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Identifier(Pos(1, 1), "key".to_string()),
            Operator(Pos(1, 5), '='),
            Operator(Pos(1, 6), '"'),
            Identifier(Pos(1, 7), "value; more value!".to_string()),
            Operator(Pos(1, 25), '"'),
            EOF,
        ]); 

        // missing trailing quote
        let s = "key =\"value; more value!";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Identifier(Pos(1, 1), "key".to_string()),
            Operator(Pos(1, 5), '='),
            Operator(Pos(1, 6), '"'),
            Identifier(Pos(1, 7), "value; more value!".to_string()),
            EOF,
        ]); 

        // inserting newline and escaping quotes
        let s = "key =\"'orbit' is an HDL \n\"\"package manager\"\"\"";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Identifier(Pos(1, 1), "key".to_string()),
            Operator(Pos(1, 5), '='),
            Operator(Pos(1, 6), '"'),
            Identifier(Pos(1, 7), "'orbit' is an HDL \n\"package manager\"".to_string()),
            Operator(Pos(2, 20), '"'),
            EOF,
        ]); 
    }

    #[test]
    fn multiple_keys() {
        let s = "\
[table]
key1 = value1
key2 = value2
";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
            Identifier(Pos(2, 1), "key1".to_string()),
            Operator(Pos(2, 6), '='),
            Identifier(Pos(2, 8), "value1".to_string()),
            EOL,
            Identifier(Pos(3, 1), "key2".to_string()),
            Operator(Pos(3, 6), '='),
            Identifier(Pos(3, 8), "value2".to_string()),
            EOL,
            EOF,
        ]);
    }

    #[test]
    fn spacing_and_eof() {
        let s = "\
[table]
key1 = value1


key2 = value2";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            EOL,
            Identifier(Pos(2, 1), "key1".to_string()),
            Operator(Pos(2, 6), '='),
            Identifier(Pos(2, 8), "value1".to_string()),
            EOL,
            EOL,
            EOL,
            Identifier(Pos(5, 1), "key2".to_string()),
            Operator(Pos(5, 6), '='),
            Identifier(Pos(5, 8), "value2".to_string()),
            EOF,
        ]);

        let s = "    [table]
  key1=  value1 ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 5), '['),
            Identifier(Pos(1, 6), "table".to_string()),
            Operator(Pos(1, 11), ']'),
            EOL,
            Identifier(Pos(2, 3), "key1".to_string()),
            Operator(Pos(2, 7), '='),
            Identifier(Pos(2, 10), "value1".to_string()),
            EOF,
        ]);
    }

    #[test]
    fn comments() {
        let s = "\
; For more information visit orbit's website.
[core]
user = chase # your name or \"alias\"! ";      
        assert_eq!(CfgLanguage::tokenize(s), vec![
            Operator(Pos(1, 1), ';'),
            Comment(Pos(1, 2), " For more information visit orbit's website.".to_string()),
            EOL,
            Operator(Pos(2, 1), '['),
            Identifier(Pos(2, 2), "core".to_string()),
            Operator(Pos(2, 6), ']'),
            EOL,
            Identifier(Pos(3, 1), "user".to_string()),
            Operator(Pos(3, 6), '='),
            Identifier(Pos(3, 8), "chase".to_string()),
            Operator(Pos(3, 14), '#'),
            Comment(Pos(3, 15), " your name or \"alias\"! ".to_string()),
            EOF,
        ]);
    }
}