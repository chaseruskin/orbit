use crate::util::anyerror::Fault;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct TemplateFile(PathBuf);

impl TemplateFile {
    pub fn new() -> Self {
        Self(PathBuf::new())
    }

    pub fn path(p: &PathBuf) -> Self {
        Self(p.to_owned())
    }

    /// Performs variable substitution on the file data
    pub fn substitute(&self, code: &HashMap<String, String>) -> Result<(), Fault> {
        // read the data
        let contents = std::fs::read_to_string(&self.0)?;
        // transform the data and write it to file
        std::fs::write(&self.0, substitute(contents, code))?;
        Ok(())
    }
}

const L_VAR_DELIMITER: char = '{';
const R_VAR_DELIMITER: char = '}';

/// Performs variable replacement on the given `text`, looking up variables in
/// the `code` to swap with their values.
fn substitute(text: String, code: &HashMap<String, String>) -> String {
    let mut result = String::new();
    
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        // check if there is a valid variable replacement
        match c {
            L_VAR_DELIMITER => {
                match gather_variable(&mut chars, c, R_VAR_DELIMITER) {
                    Ok(r) => {
                        // remove delimiters and surrounding whitespace to get key name
                        let key = &r[2..r.len()-2].trim();
                        // look up the key in the code book
                        match code.get(*key) {
                            Some(value) => result.push_str(value),
                            None => result.push_str(&r),
                        }
                    }
                    Err(e) => result.push_str(&e),
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// Builds a variable following the syntax `c0c0*c_nc_n`.
/// 
/// Assumes the first token was already consumed and is passed as `c0`.
/// 
/// Errors if the syntax is not fulfilled. 
fn gather_variable<T: Iterator<Item=char>>(chars: &mut T, c0: char, c_n: char) -> Result<String, String> {
    let mut var = String::from(c0);
    let mut last: Option<char> = None;
    // verify next character is also `c0`
    if let Some(c) = chars.next() {
        var.push(c);
        if c != c0 { return Err(var) }
        last = Some(c);
    }
    // collect remaining characters until finding double cN occurrences 
    while let Some(c) = chars.next() {
        var.push(c);
        if c == c_n {
            // check if the last char was also `cN`
            if last.unwrap() == c_n {
                return Ok(var)
            }
        }
        last = Some(c);
    }
    // never was able to close the variable
    Err(var)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::*;

    /// Internal helper test `fn` to generate a sample code book for looking up variables.
    fn create_code() -> HashMap<String, String> {
        let mut code = HashMap::new();
        code.insert("orbit.name".to_owned(), "gates".to_owned());
        code.insert("orbit.library".to_owned(), "rary".to_owned());
        code.insert("orbit.place".to_owned(), "bar".to_owned());
        code
    }

    #[test]
    fn gather_var() {
        let text = "{ variable }}";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Ok("{{ variable }}".to_owned()));

        let text = "{ variable }";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{{ variable }".to_owned()));

        let text = "variable }";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{v".to_owned()));

        let text = "{variable } } ";
        assert_eq!(gather_variable(&mut text.chars(), '{', '}'), Err("{{variable } } ".to_owned()));
    }

    #[test]
    fn replace_variables() {
        let text = "The quick brown fox jumped over the lazy {{ orbit.name }}.";
        let code = create_code();
        assert_eq!(substitute(text.to_owned(), &code), "The quick brown fox jumped over the lazy gates.".to_owned());

        let text = "A duck, a bear, and a {{ animal }} walk into a {{  orbit.place   }}...";
        let code = create_code();
        assert_eq!(substitute(text.to_owned(), &code), "A duck, a bear, and a {{ animal }} walk into a bar...".to_owned());
    }
}