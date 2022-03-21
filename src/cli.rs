//! File     : cli.rs
//! Abstract :
//!     The command-line interface parses user's requests into program code.

use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct Cli {
    positionals: Vec<Option<String>>,
    options: HashMap<String, Option<Param>>,
}

#[derive(Debug, PartialEq)]
enum Param {
    Direct(String),
    Indirect(usize),
}

impl Cli {
    pub fn new<T: Iterator<Item=String>>(cla: T) -> Cli {
        // skip the program's name
        let mut cla = cla.skip(1).peekable();
        let mut options = HashMap::new();
        let mut positionals = Vec::new();
        while let Some(arg) = cla.next() {
            if arg.starts_with("--") {
                // direct- detect if needs to split on first '=' sign
                if let Some((opt, param)) = arg.split_once('=') {
                    options.insert(opt.to_owned(), Some(Param::Direct(param.to_owned())));
                // indirect- peek if the next arg is the param to the current option
                } else if let Some(trailing) = cla.peek() {
                    if trailing.starts_with("--") {
                        options.insert(arg, None);
                    } else {
                        options.insert(arg, Some(Param::Indirect(positionals.len())));
                        positionals.push(cla.next());
                    }
                // none- no param was supplied to current option
                } else {
                    options.insert(arg, None);
                }
            } else {
                positionals.push(Some(arg));
            }
        }
        Cli {
            positionals: positionals,
            options: options,
        }
    }

    /// pop off the next positional in the list
    /// ### Errors
    /// - no valid entries left
    /// - failure to cast to `T`
    pub fn next_positional<T: FromStr + std::fmt::Debug>(&mut self) -> Result<T, CliError>
        where <T as std::str::FromStr>::Err: std::fmt::Debug {
        // find the first non-None value
        for p in &mut self.positionals {
            if p.is_some() {
                let result = p.as_ref().unwrap().parse::<T>().unwrap();
                // err: failed to cast to T
                *p = None;
                return Ok(result);
            }
        }
        todo!() // err: missing positional
    }

    pub fn get_flag(&mut self, opt: &str) -> Result<bool, CliError> {
            // check if it is in the map
            let val = self.options.get(opt);
            // user did not provide the flag
            if val.is_none() {
                return Ok(false);
            }
            // investigate if user provided a param for the flag
            if let Some(p) = val.unwrap() {
                match p {
                    Param::Direct(_) => {
                        // err: cannot have a value
                        todo!()
                    },
                    _ => {
                        Ok(true)
                    }
                }
            // user only raised flag
            } else {
                return Ok(true)
            }
    }

    /// query for a particular option and get it's value
    fn get_option<T: FromStr + std::fmt::Debug>(&mut self, opt: &str) -> Result<Option<T>, CliError>
        where <T as std::str::FromStr>::Err: std::fmt::Debug {
        // check if it is in the map
        let val = self.options.get(opt);
        // user did not provide option -- :todo: provide default if available
        if val.is_none() {
            return Ok(None);
        }
        // invesigate if the user provided a param for the option
        if let Some(p) = val.unwrap() {
            match p {
                Param::Direct(s) => {
                    // cast to T
                    Ok(Some(s.parse::<T>().unwrap()))
                },
                Param::Indirect(i) => {
                    // `i` is verified to be within size of vec
                    let p = self.positionals.get(*i).unwrap();
                    if p.is_none() {
                        // ? panic: options should be listed first in subcommand
                        todo!() // err: require a value
                    }
                    let result = p.as_ref().unwrap().parse::<T>().unwrap();
                    // perform a swap on the data unless it has already been used up
                    self.positionals[*i] = None;
                    Ok(Some(result))
                }
            }
        } else {
            todo!() // err: require a value
        }
    }
}

struct Arg {
    name: String,
}

enum Optional {
    AsParam(String),
    AsFlag(String),
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    BadType,
}

// order: options w/ params, positionals, options

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_args() {
        let args = vec![
            "orbit",
        ].into_iter().map(|s| s.to_owned());
        let cli = Cli::new(args);

        assert_eq!(cli, Cli {
            positionals: Vec::new(),
            options: HashMap::new(),
        });
    }

    #[test]
    fn options() {
        let args = vec![
            "orbit",
            "--version",
            "--help",
        ].into_iter().map(|s| s.to_owned());
        let cli = Cli::new(args);

        let mut opts = HashMap::new();
        opts.insert("--version".to_owned(), None);
        opts.insert("--help".to_owned(), None);

        assert_eq!(cli, Cli {
            positionals: Vec::new(),
            options: opts,
        });
    }

    #[test]
    fn options_with_values() {
        let args = vec![
            "orbit",
            "--path",
            "C:/Users/chase/hdl",
            "--verbose=2",
        ].into_iter().map(|s| s.to_owned());
        let mut opts = HashMap::new();
        opts.insert("--path".to_owned(), Some(Param::Indirect(0)));
        opts.insert("--verbose".to_owned(), Some(Param::Direct("2".to_owned())));
        
        let cli = Cli::new(args);
        assert_eq!(cli, Cli {
            positionals: vec![
                Some("C:/Users/chase/hdl".to_owned()),
            ],
            options: opts,
        });
    }

    #[test]
    fn positional() {
        let args = vec![
            "orbit",
            "new",
            "rary.gates"
        ].into_iter().map(|s| s.to_owned());
        let cli = Cli::new(args);

        assert_eq!(cli, Cli {
            positionals: vec![
                Some("new".to_owned()),
                Some("rary.gates".to_owned()),
            ],
            options: HashMap::new(),
        });
    }

    #[test]
    fn query_cli() {
        // $ orbit new --path C:/Users/chase rary.gates --verbose=2
        let mut opts = HashMap::new();
        opts.insert("--path".to_owned(), Some(Param::Indirect(1)));
        opts.insert("--verbose".to_owned(), Some(Param::Direct("2".to_owned())));
        let mut cli = Cli {
            positionals: vec![
                Some("new".to_owned()),
                Some("C:/Users/chase".to_owned()),
                Some("rary.gates".to_owned()),
            ],
            options: opts,
        };

        assert_eq!(cli.get_option("--verbose"), Ok(Some(2)));
        assert_eq!(cli.get_option("--path"), Ok(Some("C:/Users/chase".to_owned())));
        assert_eq!(cli.next_positional(), Ok("new".to_owned()));
        assert_eq!(cli.next_positional(), Ok("rary.gates".to_owned()));

        let args = vec![
            "orbit", "--version", "get", "gates::nor_gate", "--code", "vhdl",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option("--code"), Ok(Some("vhdl".to_string())));
        assert_eq!(cli.next_positional(), Ok("get".to_owned()));
        assert_eq!(cli.next_positional(), Ok("gates::nor_gate".to_owned()));
        assert_eq!(cli.get_flag("--version"), Ok(true));

        let args = vec![
            "orbit", "--path", "c:/users/chase", "--stats", "info", "--help",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option("--path"), Ok(Some("c:/users/chase".to_string())));
        assert_eq!(cli.next_positional(), Ok("info".to_owned()));
        assert_eq!(cli.get_flag("--stats"), Ok(true));
        assert_eq!(cli.get_flag("--version"), Ok(false));
        assert_eq!(cli.get_flag("--help"), Ok(true));
    }
}