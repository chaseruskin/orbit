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
    // build up vector as parameters are queried to generate a list of valid params 
    // to later compute edit distance if an unknown argument was entered for a command
    // known_params: Vec<String>,
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

    /// Pop off the next positional in the provided order.
    pub fn next_positional<T: FromStr + std::fmt::Debug>(&mut self) -> Result<T, CliError>
        where <T as std::str::FromStr>::Err: std::fmt::Debug {
        // :todo: refactor using iterators
        self.past_opts = true;
        for p in &mut self.positionals {        
            // find the first non-None value
            if p.is_some() {
                if let Ok(r) = p.take().unwrap().parse::<T>() {
                    return Ok(r)
                } else {
                    todo!("failed to cast to T for positional")
                }
            }
        }
        // found zero available arguments
        Err(CliError::MissingPositional)
    }

    // :todo: make better
    pub fn set_past(&mut self, b: bool) {
        self.past_opts = b;
    }

    /// Query if a flag was raised. Returns errors if a direct value was given or
    /// if the flag was raised multiple times
    pub fn get_flag(&mut self, flag: Flag) -> Result<bool, CliError> {
        // // check if it is in the map
        if let Some(mut val) = self.options.remove(&("--".to_owned()+flag.0)) {
            // raise error if there is an attached option to the flag
            if val.len() > 1 {
                // err: duplicate values
                Err(CliError::DuplicateOptions)
            } else {
                match val.pop().unwrap() {
                    Some(p) => match p {
                        Param::Direct(s) => Err(CliError::UnexpectedValue(flag.0.to_owned(), s)),
                        Param::Indirect(_) => Ok(true),
                    }
                    None => Ok(true),
                }
            }
        // flag was not found in options map
        } else {
            Ok(false)
        }
    }

    /// Ensure there are no unused/unchecked arguments. Results in error if
    /// `options` is not empty or `positionals` has a non-None value.
    pub fn is_clean(&self) -> Result<(), CliError> {
        if self.options.is_empty() != true {
            Err(CliError::UnexpectedArg(self.options.keys().next().unwrap().to_owned()))
        } else if let Some(Some(a)) = self.positionals.iter().find(|f| f.is_some()) {
            Err(CliError::UnexpectedArg(a.to_owned()))
        } else {
            Ok(())
        }
    }

    /// Retuns the vector of leftover arguments to pass to internal command. Also
    /// updates removing the `--` flag from the `options` map.
    pub fn get_remainder(&mut self) -> &Vec::<String> {
        self.options.remove("--");
        return &self.remainder
    }

    /// Query for a particular option and get it's value.
    /// To set a default value, chain `.or()` to this function call.
    pub fn get_option<T: FromStr + std::fmt::Debug>(&mut self, opt: Optional) -> Result<Option<T>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        // check if it is in the map (pull from map)
        if let Some(mut m) = self.options.remove(opt.0) { 
            if m.len() > 1 {
                Err(CliError::DuplicateOptions)
            // investigate if the user provided a param for the option
            } else if let Some(p) = m.pop().unwrap() {
                Ok(Some(self.parse_param(p)?))
            } else {
                Err(CliError::ExpectingValue(opt.0.to_owned()))
            }
        } else { 
            Ok(None)
        }
    }

    /// Handle updating the positional vector depending on if a param was direct
    /// or indirect.
    fn parse_param<T: FromStr + std::fmt::Debug>(&mut self, p: Param) -> Result<T, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        match p {
            Param::Direct(s) => {
                if let Ok(c) = s.parse::<T>() {
                    Ok(c)
                } else {
                    todo!("handle parse error")
                }
            }
            Param::Indirect(i) => {
                // `i` is verified to be within size of vec
                let p = &mut self.positionals[i];
                // perform a swap on the data unless it has already been used up
                if let Ok(c) = p.take().expect("value was stolen from option").parse::<T>() {
                    Ok(c)
                } else {
                    todo!("handle parse error");
                }
            }
        }
   }

    /// Query for a particular option and return back all values provided
    pub fn get_option_vec<T: FromStr + std::fmt::Debug>(&mut self, opt: &str) -> Result<Option<Vec<T>>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        if let Some(m) = self.options.remove(opt) {
            let mut res = Vec::<T>::with_capacity(m.len());
            let mut m = m.into_iter();
            while let Some(e) = m.next() {
                if let Some(p) = e {
                    res.push(self.parse_param(p)?);
                } else {
                    return Err(CliError::ExpectingValue(opt.to_owned()));
                }
            }
            Ok(Some(res))
        } else {
            // option was not provided by user, return None
            Ok(None)
        }
    }
}

pub struct Positional<'a>(&'a str);

impl Display for Positional<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "<{}>", self.0)
    }
}

pub struct Flag<'a>(pub &'a str);

impl Display for Flag<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "--{}", self.0)
    }
}

pub struct Optional<'a>(pub &'a str);

impl Display for Optional<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "--{} <{}>", self.0, self.0)
    }
}

pub enum Arg<'a> {
    Positional(Positional<'a>),
    Flag(Flag<'a>),
    Optional(Optional<'a>),
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    BadType,
    MissingPositional,
    DuplicateOptions,
    ExpectingValue(String),
    UnexpectedValue(String, String),
    UnexpectedArg(String),
}

impl Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use CliError::*;
        match self {
            BadType => write!(f, "bad type conversion from string"),
            MissingPositional => write!(f, "missing positional"),
            DuplicateOptions => write!(f, "duplicate options"),
            ExpectingValue(x) => write!(f, "option \"{}\" expects a value but none was supplied", x),
            UnexpectedValue(x, s) => write!(f, "flag \"{}\" cannot accept values but one was supplied \"{}\"", x, s),
            UnexpectedArg(s) => write!(f, "unknown argument \"{}\"", s),
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

        assert_eq!(cli.get_flag(Flag("lib")), Ok(true));
        assert_eq!(cli.get_option::<String>(Optional("--log")), Ok(None));
        // undesired here... options must be evaluated before positionals
        assert_eq!(cli.next_positional(), Ok("general:editor=code".to_owned()));
        assert_eq!(cli.next_positional(), Ok("new".to_owned()));
        // panic occurs on first call to an option once beginning to read positionals
        assert_eq!(cli.get_option(Optional("--config")), Ok(Some("general:editor=code".to_owned())));
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

        assert_eq!(cli.get_option::<String>(Optional("--path")), Ok(None));
        assert_eq!(cli.get_option::<String>(Optional("--log")), Ok(None));
        assert_eq!(cli.next_positional(), Ok("build".to_owned()));
        assert_eq!(cli.next_positional(), Ok("quartus".to_owned()));
        // verify there are no more positionals
        assert!(cli.next_positional::<String>().is_err());
        assert_eq!(cli.get_flag(Flag("plan")), Ok(true));
        assert_eq!(cli.get_flag(Flag("version")), Ok(false));
        assert_eq!(cli.get_flag(Flag("help")), Ok(false));
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

        assert_eq!(cli.get_option(Optional("--verbose")), Ok(Some(2)));
        assert_eq!(cli.get_option(Optional("--path")), Ok(Some("C:/Users/chase".to_owned())));
        assert_eq!(cli.next_positional(), Ok("new".to_owned()));
        assert_eq!(cli.next_positional(), Ok("rary.gates".to_owned()));

        let args = vec![
            "orbit", "--version", "get", "gates::nor_gate", "--code", "vhdl",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional("--code")), Ok(Some("vhdl".to_string())));
        assert_eq!(cli.next_positional(), Ok("get".to_owned()));
        assert_eq!(cli.next_positional(), Ok("gates::nor_gate".to_owned()));
        assert_eq!(cli.get_flag(Flag("version")), Ok(true));
        assert!(cli.is_clean().is_ok());

        let args = vec![
            "orbit", "--path", "c:/users/chase", "--stats", "info", "--help",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional("--path")), Ok(Some("c:/users/chase".to_string())));
        assert_eq!(cli.next_positional(), Ok("info".to_owned()));
        assert_eq!(cli.get_flag(Flag("stats")), Ok(true));
        assert_eq!(cli.get_flag(Flag("version")), Ok(false));
        assert_eq!(cli.get_flag(Flag("help")), Ok(true));
        assert!(cli.is_clean().is_ok());
    }

    #[test]
    fn unknown_args() {
        let args = vec![
            "orbit", "--version", "--unknown",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);
        // command only expects these two flags
        assert_eq!(cli.get_flag(Flag("version")), Ok(true));
        assert_eq!(cli.get_flag(Flag("help")), Ok(false));
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