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
}

fn tokenize(s: &str) -> Vec::<Token> {
    let mut tokens = Vec::new();
    let mut line: Line = 1;
    let mut col: Col = 0;
    let mut buf: String = String::new();

    for c in s.chars() {
        col += 1;
        match c {
            ']' | '[' | '=' => {
                if buf.len() > 0 {
                    tokens.push(Token::Identifier(Pos(line, col-buf.len()), buf.clone()));
                    buf = String::new();
                }
                tokens.push(Token::Operator(Pos(line, col), c));
            }
            '\n' => {
                if buf.len() > 0 {
                    tokens.push(Token::Identifier(Pos(line, col-buf.len()), buf.clone()));
                    buf = String::new();
                }
                line += 1;
                col = 0;
            }
            ' ' => {
                if buf.len() > 0 {
                    tokens.push(Token::Identifier(Pos(line, col-buf.len()), buf.clone()));
                    buf = String::new();
                }
            }
            _ => {
                buf.push(c);
            }
        }
    }

    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lexer() {
        use Token::*;
        let s = "\
[table]
key = value
";      
        let tokens = tokenize(s);
        assert_eq!(tokens, vec![
            Operator(Pos(1, 1), '['),
            Identifier(Pos(1, 2), "table".to_string()),
            Operator(Pos(1, 7), ']'),
            Identifier(Pos(2, 1), "key".to_string()),
            Operator(Pos(2, 5), '='),
            Identifier(Pos(2, 7), "value".to_string()),
        ]);
    }
}