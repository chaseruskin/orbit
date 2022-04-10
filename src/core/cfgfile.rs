//! File     : cfgfile.rs
//! Abstract :
//!     A `cfgfile` is the main file format used to store data for Orbit. It
//!     resembles a ini-like syntax and structure composed of "tables" 
//!     (sections) and "fields" (key-value pairs).


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
    EOF,
}

struct CfgLanguage {
    tokens: Vec::<Token>,
}

enum CfgState {
    COMMENT,
    QUOTE,
    NORMAL,
}

impl CfgLanguage {
    fn new() -> Self {
        CfgLanguage { tokens: Vec::new() }
    }

    fn complete_token(&mut self, s: &mut String, p: &Pos) {
        if s.is_empty() == false {
            self.tokens.push(Token::Identifier(Pos(p.0, p.1-s.len()), s.trim().to_string()));
            s.clear();
        }
    }
    
    /// Given some text `s`, tokenize it according the cfg language.
    fn tokenize(&mut self, s: &str) {
        let mut line: Line = 1;
        let mut col: Col = 0;
        let mut buf: String = String::new();
        let mut state = CfgState::NORMAL;

        for c in s.chars() {
            col += 1;
            match state {
                CfgState::COMMENT => {
                    match c {
                        '\n' => {
                            self.tokens.push(Token::Comment(Pos(line, col-buf.len()), buf.to_string()));
                            buf.clear();
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
                            self.complete_token(&mut buf, &Pos(line, col));
                            self.tokens.push(Token::Operator(Pos(line, col), c));
                            state = CfgState::COMMENT;
                        }
                        ']' | '[' | '=' | '\"' | '\'' => {
                            self.complete_token(&mut buf, &Pos(line, col));
                            self.tokens.push(Token::Operator(Pos(line, col), c));
                        }
                        '\n' => {
                            self.complete_token(&mut buf, &Pos(line, col));
                            line += 1;
                            col = 0;
                        }
                        _ => {
                            if (c.is_whitespace() == false) || (buf.is_empty() == false) {
                                buf.push(c);
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        // final check to ensure emptying the buffer
        match state {
            CfgState::COMMENT => {
                self.tokens.push(Token::Comment(Pos(line, col-buf.len()+1), buf.to_string()));
            },
            // col+1 because missed ending
            CfgState::NORMAL => {
                self.complete_token(&mut buf, &Pos(line, col+1));
            },
            _ => (),
        }
        self.tokens.push(Token::EOF);
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    #[test]
    fn basic_lexer() {
        let s = "\
[table]
key = value
";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Identifier(Pos(2, 7), "value".to_string()),
            EOF,
        ]);

        let s = "\
[table]
key = place the value here
";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Identifier(Pos(2, 7), "place the value here".to_string()),
            EOF,
        ]);

        let s = "\
[table]
key = \"value\"
jot = 'notes'
";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Operator(Pos(2, 7), '"'),
            Identifier(Pos(2, 8), "value".to_string()),
            Operator(Pos(2, 13), '"'),
            Identifier(Pos(3, 1), "jot".to_string()),
            Operator(Pos(3, 5), '='),
            Operator(Pos(3, 7), '\''),
            Identifier(Pos(3, 8), "notes".to_string()),
            Operator(Pos(3, 13), '\''),
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
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key1".to_string()),
            Operator(Pos(2, 6), '='),
            Identifier(Pos(2, 8), "value1".to_string()),
            Identifier(Pos(3, 1), "key2".to_string()),
            Operator(Pos(3, 6), '='),
            Identifier(Pos(3, 8), "value2".to_string()),
            EOF,
        ]);
    }

    #[test]
    fn spacing_and_eof() {
        let s = "\
[table]
key1 = value1


key2 = value2";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key1".to_string()),
            Operator(Pos(2, 6), '='),
            Identifier(Pos(2, 8), "value1".to_string()),
            Identifier(Pos(5, 1), "key2".to_string()),
            Operator(Pos(5, 6), '='),
            Identifier(Pos(5, 8), "value2".to_string()),
            EOF,
        ]);

        let s = "    [table]
  key1=  value1 ";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 5), '['),
            Identifier(Pos(1, 6), "table".to_string()),
            Operator(Pos(1, 11), ']'),
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
user = chase # your name! ";      
        let mut cfg = CfgLanguage::new();
        cfg.tokenize(s);
        assert_eq!(cfg.tokens, vec![
            Operator(Pos(1, 1), ';'),
            Comment(Pos(1, 2), " For more information visit orbit's website.".to_string()),
            Operator(Pos(2, 1), '['),
            Identifier(Pos(2, 2), "core".to_string()),
            Operator(Pos(2, 6), ']'),
            Identifier(Pos(3, 1), "user".to_string()),
            Operator(Pos(3, 6), '='),
            Identifier(Pos(3, 8), "chase".to_string()),
            Operator(Pos(3, 14), '#'),
            Comment(Pos(3, 15), " your name! ".to_string()),
            EOF,
        ]);
    }
}