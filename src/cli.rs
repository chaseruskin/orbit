//! File     : cli.rs
//! Abstract :
//!     The command-line interface parses user's requests into program code.
//! 
//! Notes    :
//! - options must be queried before positionals

use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;
use std::error::Error;
use std::fmt::Display;

type Value = Vec::<Option<Param>>;

#[derive(Debug, PartialEq)]
pub struct Cli {
    positionals: Vec<Option<String>>,
    options: HashMap<String, Value>,
    remainder: Vec::<String>,
    past_opts: bool,
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
        let mut options = HashMap::<String, Value>::new();
        let mut positionals = Vec::new();
        // appending an element behind the key (ensuring a vector always exist)
        let mut enter = |k, v| {
            options
                .entry(k)
                .or_insert(Vec::new())
                .push(v);
        };
        // iterate through available arguments
        while let Some(arg) = cla.next() {
            if arg == "--" {
                enter(arg, None);
                break;
            } else if arg.starts_with("--") {
                // direct- detect if needs to split on first '=' sign
                if let Some((opt, param)) = arg.split_once('=') {
                    enter(opt.to_owned(), Some(Param::Direct(param.to_owned())));
                // indirect- peek if the next arg is the param to the current option
                } else if let Some(trailing) = cla.peek() {
                    if trailing.starts_with("--") {
                        enter(arg, None);
                    } else {
                        enter(arg, Some(Param::Indirect(positionals.len())));
                        positionals.push(cla.next());
                    }
                // none- no param was supplied to current option
                } else {
                    enter(arg, None);
                }
            } else {
                positionals.push(Some(arg));
            }
        }
        Cli {
            positionals: positionals,
            options: options,
            remainder: cla.collect(),
            past_opts: false,
        }
    }

    /// pop off the next positional in the list
    /// ### Errors
    /// - no valid entries left
    /// - failure to cast to `T`
    pub fn next_positional<T: FromStr + std::fmt::Debug>(&mut self) -> Result<T, CliError>
        where <T as std::str::FromStr>::Err: std::fmt::Debug {
        self.past_opts = true;
        // find the first non-None value
        for p in &mut self.positionals {
            if p.is_some() {
                // :todo: err: failed to cast to T
                let result = p.as_ref().unwrap().parse::<T>().unwrap();
                *p = None;
                return Ok(result);
            }
        }
        // err: missing positional
        Err(CliError::ExpectingPositional)
    }

    pub fn get_flag(&mut self, opt: &str) -> Result<bool, CliError> {
        // check if it is in the map
        let val = self.options.remove(opt);
        // user did not provide the flag
        let mut val = if let None = val {
            return Ok(false);
        } else {
            val.unwrap()
        };
        let element = val.pop().unwrap();
        if val.is_empty() == false {
            // err: duplicate values
            return Err(CliError::DuplicateOptions);
        }
        // investigate if user provided a param for the flag
        if let Some(p) = element {
            match p {
                Param::Direct(_) => {
                    // err: cannot have a value
                    return Err(CliError::UnexpectedValue);
                },
                Param::Indirect(_) => {
                    Ok(true)
                }
            }
        // user only raised flag
        } else {
            Ok(true)
        }
    }

    /// Ensure there are no unused/unchecked arguments. Results in error if
    /// `options` is not empty or `positionals` has a non-None value.
    pub fn is_clean(&self) -> Result<(), CliError> {
        if self.options.is_empty() != true {
            return Err(CliError::UnexpectedArg);
        }
        if self.positionals.iter().find(|f| f.is_some()).is_some() {
            return Err(CliError::UnexpectedArg);
        }
        Ok(())
    }

    /// Retuns the vector of leftover arguments to pass to internal command. Also
    /// updates removing the `--` flag from the `options` map.
    pub fn get_remainder(&mut self) -> &Vec::<String> {
        self.options.remove("--");
        return &self.remainder
    }

    /// Query for a particular option and get it's value.
    fn get_option<T: FromStr + std::fmt::Debug>(&mut self, opt: &str) -> Result<Option<T>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        // check if it is in the map (pull from map)
        let val = self.options.remove(opt);
        // user did not provide option -- :todo: provide default if available
        let mut val = if let None = val {
            return Ok(None);
        } else {
            val.unwrap()
        };
        let element = val.pop().unwrap();
        if val.is_empty() == false {
            // err: duplicate values
            return Err(CliError::DuplicateOptions);
        }
        // investigate if the user provided a param for the option
        if let Some(p) = element {
            match p {
                Param::Direct(s) => {
                    // cast to T
                    Ok(Some(s.parse::<T>().unwrap()))
                },
                Param::Indirect(i) => {
                    // `i` is verified to be within size of vec; safe to unwrap
                    let p = self.positionals.get(i).unwrap();
                    if p.is_none() {
                        // panic: options should be listed first in subcommand
                        panic!("value was stolen from option {}", opt)
                    }
                    let result = p.as_ref().unwrap().parse::<T>().unwrap();
                    // perform a swap on the data unless it has already been used up
                    self.positionals[i] = None;
                    Ok(Some(result))
                }
            }
        } else {
            // err: require a value
            Err(CliError::ExpectingValue)
        }
    }

    /// Query for a particular option and return back all values as list
    fn get_option_vec<T: FromStr + std::fmt::Debug>(&mut self, opt: &str) -> Result<Option<Vec<T>>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        let val = self.options.remove(opt);
        // option was not provided by user, return None
        if let None = val {
            return Ok(None);
        }
        let mut res = Vec::<T>::with_capacity(val.as_ref().unwrap().len());
        let mut val = val.unwrap().into_iter();
        while let Some(e) = val.next() {
            if let Some(p) = e {
                match p {
                    Param::Direct(s) => {
                        res.push(s.parse::<T>().unwrap()); // :todo: handle err
                    }
                    Param::Indirect(i) => {
                        // `i` is verified to be within size of vec
                        let p = self.positionals.get(i).unwrap();
                        if p.is_none() {
                            // panic: options should be listed first in subcommand
                            panic!("value was stolen from option {}", opt)
                        }
                        let result = p.as_ref().unwrap().parse::<T>().unwrap();
                        // perform a swap on the data unless it has already been used up
                        self.positionals[i] = None;
                        res.push(result);
                    }
                }
            } else {
                // err: a option was provided with no value
                return Err(CliError::ExpectingValue);
            }
        }
        Ok(Some(res))
    }
}

// struct Opt<'a, T>(&'a str, Option<T>);

// enum Arg<'a> {
//     Positional(&'a str),
//     Optional(Opt<'a, T>),
//     Flag(&'a str),
//     MultipleOpt(&'a str),
// }

#[derive(Debug, PartialEq)]
pub enum CliError {
    BadType,
    ExpectingPositional,
    DuplicateOptions,
    ExpectingValue,
    UnexpectedValue,
    UnexpectedArg,
}

impl Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use CliError::*;
        match self {
            BadType => write!(f, "bad type conversion from string"),
            ExpectingPositional => write!(f, "expecting positional"),
            DuplicateOptions => write!(f, "duplicate options"),
            ExpectingValue => write!(f, "expecting value"),
            UnexpectedValue => write!(f, "unexpected value"),
            UnexpectedArg => write!(f, "unexpected argument"),
        }
    }
}

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
            remainder: Vec::new(),
            past_opts: false,
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
        opts.insert("--version".to_owned(), vec![None]);
        opts.insert("--help".to_owned(), vec![None]);

        assert_eq!(cli, Cli {
            positionals: Vec::new(),
            options: opts,
            remainder: Vec::new(),
            past_opts: false,
        });
    }

    #[test]
    #[should_panic = "options must be evaluated before positionals"]
    fn options_after_positionals() {
        let args = vec![
            "orbit", "--config", "general:editor=code", "new", "rary.gates", "--open", "--lib"
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_flag("--lib"), Ok(true));
        assert_eq!(cli.get_option::<String>("--log"), Ok(None));
        // undesired here... options must be evaluated before positionals
        assert_eq!(cli.next_positional(), Ok("general:editor=code".to_owned()));
        assert_eq!(cli.next_positional(), Ok("new".to_owned()));
        // panic occurs on first call to an option once beginning to read positionals
        assert_eq!(cli.get_option::<String>("--config"), Ok(Some("general:editor=code".to_owned())));
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
        opts.insert("--path".to_owned(), vec![Some(Param::Indirect(0))]);
        opts.insert("--verbose".to_owned(), vec![Some(Param::Direct("2".to_owned()))]);
        
        let cli = Cli::new(args);
        assert_eq!(cli, Cli {
            positionals: vec![
                Some("C:/Users/chase/hdl".to_owned()),
            ],
            options: opts,
            remainder: Vec::new(),
            past_opts: false,
        });
    }

    #[test]
    fn passing_args_with_double_dash() {
        let args = vec![
            "orbit", "build", "quartus", "--plan", "--", "synthesize", "--log", "./quartus.log"
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option::<String>("--path"), Ok(None));
        assert_eq!(cli.get_option::<String>("--log"), Ok(None));
        assert_eq!(cli.next_positional(), Ok("build".to_owned()));
        assert_eq!(cli.next_positional(), Ok("quartus".to_owned()));
        // verify there are no more positionals
        assert!(cli.next_positional::<String>().is_err());
        assert_eq!(cli.get_flag("--plan"), Ok(true));
        assert_eq!(cli.get_flag("--version"), Ok(false));
        assert_eq!(cli.get_flag("--help"), Ok(false));
        // these arguments are passed to internally called command
        assert_eq!(cli.get_remainder(), &vec!["synthesize", "--log", "./quartus.log"]);
        assert!(cli.is_clean().is_ok());
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
            remainder: Vec::new(),
            past_opts: false,
        });
    }

    #[test]
    fn query_cli() {
        // $ orbit new --path C:/Users/chase rary.gates --verbose=2
        let mut opts = HashMap::new();
        opts.insert("--path".to_owned(), vec![Some(Param::Indirect(1))]);
        opts.insert("--verbose".to_owned(), vec![Some(Param::Direct("2".to_owned()))]);
        let mut cli = Cli {
            positionals: vec![
                Some("new".to_owned()),
                Some("C:/Users/chase".to_owned()),
                Some("rary.gates".to_owned()),
            ],
            options: opts,
            remainder: Vec::new(),
            past_opts: false,
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
        assert!(cli.is_clean().is_ok());

        let args = vec![
            "orbit", "--path", "c:/users/chase", "--stats", "info", "--help",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option("--path"), Ok(Some("c:/users/chase".to_string())));
        assert_eq!(cli.next_positional(), Ok("info".to_owned()));
        assert_eq!(cli.get_flag("--stats"), Ok(true));
        assert_eq!(cli.get_flag("--version"), Ok(false));
        assert_eq!(cli.get_flag("--help"), Ok(true));
        assert!(cli.is_clean().is_ok());
    }

    #[test]
    fn unknown_args() {
        let args = vec![
            "orbit", "--version", "--unknown",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);
        // command only expects these two flags
        assert_eq!(cli.get_flag("--version"), Ok(true));
        assert_eq!(cli.get_flag("--help"), Ok(false));
        // --unknown was not caught
        assert!(cli.is_clean().is_err());
    }

    #[test]
    fn dupe_options() {
        let args = vec![
            "orbit", "--config", "general:editor=code", "--config", "general:author=chase",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option_vec("--config"), Ok(Some(vec![
            "general:editor=code".to_string(),
            "general:author=chase".to_string(),
        ])));
    }
}