//! File     : cli.rs
//! Abstract :
//!     The command-line interface parses user's requests into program code.
//! 
//! Notes    :
//! - options must be queried before positionals
//! - parameters must be provided only after their respective subcommands

use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;
use std::error::Error;
use std::fmt::Display;

type Value = Vec::<Option<Param>>;
type Index = usize;

#[derive(Debug, PartialEq)]
struct ParamArg(Index, Value);

#[derive(Debug, PartialEq)]
struct PosArg(Index, String);

#[derive(Debug, PartialEq)]
pub struct Cli {
    positionals: Vec<Option<PosArg>>,
    options: HashMap<String, ParamArg>,
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

// #[derive(Debug, PartialEq)]
// enum PosArg {
//     Ambigious(usize, String),
//     Clear(usize, String),
// }

// impl PosArg {
//     fn reveal(&self) -> &String {
//         match self {
//             Self::Ambigious(_, a) => a,
//             Self::Clear(_, a) => a,
//         }
//     }

//     fn reveal_mut(&mut self) -> &mut String {
//         match self {
//             Self::Ambigious(_, a) => a,
//             Self::Clear(_, a) => a,
//         }
//     }
// }

impl Cli {
    pub fn new<T: Iterator<Item=String>>(cla: T) -> Cli {
        // skip the program's name
        let mut cla = cla.skip(1).enumerate().peekable();
        let mut options = HashMap::<String, ParamArg>::new();
        let mut positionals = Vec::new();
        // appending an element behind the key (ensuring a vector always exist)
        let mut enter = |k, v, i| {
            options
                .entry(k)
                .or_insert(ParamArg(i, Vec::new())).1
                .push(v);
        };
        // iterate through available arguments
        while let Some((i, arg)) = cla.next() {
            if arg == "--" {
                enter(arg, None, i);
                break;
            } else if arg.starts_with("--") {
                // direct- detect if needs to split on first '=' sign
                if let Some((opt, param)) = arg.split_once('=') {
                    enter(opt.to_owned(), Some(Param::Direct(param.to_owned())), i);
                // indirect- peek if the next arg is the param to the current option
                } else if let Some((_, trailing)) = cla.peek() {
                    if trailing.starts_with("--") {
                        enter(arg, None, i);
                    } else {
                        enter(arg, Some(Param::Indirect(positionals.len())), i);
                        match cla.next() {
                            Some((k, fa)) => positionals.push(Some(PosArg(k, fa))),
                            None => positionals.push(None),
                        };
                    }
                // none- no param was supplied to current option
                } else {
                    enter(arg, None, i);
                }
            } else {
                positionals.push(Some(PosArg(i, arg)));
            }
        }
        Cli {
            positionals: positionals,
            options: options,
            remainder: cla.map(|(_, v)| v).collect(),
            past_opts: false,
        }
    }

    pub fn next_command<T: crate::command::Dispatch>(&mut self, arg: Positional) -> Result<Option<Box<dyn crate::command::Command>>, CliError> {
        // continually skip values that could be indirect with flags (or that fail)
        let pos_it = self.positionals
            .iter()
            .find(|p| p.is_some());

        let i = if let Some(a) = pos_it {
            a.as_ref().unwrap().0
        } else {
            return Ok(None);
        };
        // :todo: add ability to offer suggestion to maybe move ooc arg after the successfully parsed subcommand
        self.is_partial_clean(i)?;
        let sub: String = self.next_positional(arg)?;
        self.past_opts = false;
        Ok(Some(T::dispatch(&sub, self)?))
    }

    /// Pop off the next positional in the provided order.
    pub fn next_positional<T: FromStr + std::fmt::Debug>(&mut self, arg: Positional) -> Result<T, CliError>
        where <T as std::str::FromStr>::Err: std::fmt::Debug {
        self.past_opts = true;
        if let Some(p) = self.positionals.iter_mut()
            .skip_while(|s| s.is_none())
            .next() {
            if let Ok(r) = p.take().unwrap().1.parse::<T>() {
                return Ok(r)
            } else {
                todo!("handle error for invalid positional value")
            }
        } else {
            // found zero available arguments
            Err(CliError::MissingPositional(arg.0.to_owned()))
        }
    }

    /// Query if a flag was raised. Returns errors if a direct value was given or
    /// if the flag was raised multiple times
    pub fn get_flag(&mut self, flag: Flag) -> Result<bool, CliError> {
        // // check if it is in the map
        if let Some(mut val) = self.options.remove(&("--".to_owned()+flag.0)) {
            // raise error if there is an attached option to the flag
            if val.1.len() > 1 {
                // err: duplicate values
                Err(CliError::DuplicateOptions(flag.0.to_owned()))
            } else {
                match val.1.pop().unwrap() {
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
    
    pub fn is_partial_clean(&self, i: usize) -> Result<(), CliError> {
        // :todo: filter out options that have a position < i
        let m = self.options.iter().find(|(_, o)| {
            o.0 <= i
        });
        if let Some(arg) = m {
            Err(CliError::OutOfContextArg(arg.0.to_string()))
        } else {
            Ok(())
        }
    }

    /// Ensure there are no unused/unchecked arguments. Results in error if
    /// `options` is not empty or `positionals` has a non-None value.
    pub fn is_clean(&self) -> Result<(), CliError> {
        // :idea: clean up until a given subcommand? then pass rest of it to subcommand data
        // :todo: filter out options that have a position < i
        if self.options.is_empty() != true {
            Err(CliError::UnexpectedArg(self.options.keys().next().unwrap().to_owned()))
        // :todo: take positonals up until i
        } else if let Some(Some(a)) = self.positionals.iter().find(|f| f.is_some()) {
            Err(CliError::UnexpectedArg(a.1.to_owned()))
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
    /// To set a default value, chain `.unwrap_or()` to this function call.
    pub fn get_option<T: FromStr + std::fmt::Debug>(&mut self, opt: Optional) -> Result<Option<T>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        // check if it is in the map (pull from map)
        if let Some(mut m) = self.options.remove(opt.0) { 
            if m.1.len() > 1 {
                Err(CliError::DuplicateOptions(opt.0.to_owned()))
            // investigate if the user provided a param for the option
            } else if let Some(p) = m.1.pop().unwrap() {
                Ok(Some(self.parse_param(p)?))
            } else {
                Err(CliError::ExpectingValue(opt.0.to_owned()))
            }
        } else { 
            Ok(None)
        }
    }

    /// Query for a particular option and return back all values provided
    pub fn get_option_vec<T: FromStr + std::fmt::Debug>(&mut self, opt: Optional) -> Result<Option<Vec<T>>, CliError>
    where <T as std::str::FromStr>::Err: std::fmt::Debug {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        if let Some(m) = self.options.remove(opt.0) {
            let mut res = Vec::<T>::with_capacity(m.1.len());
            let mut m = m.1.into_iter();
            while let Some(e) = m.next() {
                if let Some(p) = e {
                    res.push(self.parse_param(p)?);
                } else {
                    return Err(CliError::ExpectingValue(opt.0.to_owned()));
                }
            }
            Ok(Some(res))
        } else {
            // option was not provided by user, return None
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
                if let Ok(c) = p.take().expect("value was stolen by positional").1.parse::<T>() {
                    Ok(c)
                } else {
                    todo!("handle parse error");
                }
            }
        }
   }
}

pub struct Positional<'a>(pub &'a str);

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
    MissingPositional(String),
    DuplicateOptions(String),
    ExpectingValue(String),
    UnexpectedValue(String, String),
    OutOfContextArg(String),
    UnexpectedArg(String),
}

impl Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use CliError::*;
        match self {
            OutOfContextArg(o) => write!(f, "invalid arg in current context \"{}\"", o),
            BadType => write!(f, "bad type conversion from string"),
            MissingPositional(p) => write!(f, "missing positional <{}>", p),
            DuplicateOptions(o) => write!(f, "duplicate options \"{}\"", o),
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
        opts.insert("--version".to_owned(), ParamArg(0, vec![None]));
        opts.insert("--help".to_owned(), ParamArg(1, vec![None]));

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
        assert_eq!(cli.next_positional(Positional("subcommand")), Ok("general:editor=code".to_owned()));
        assert_eq!(cli.next_positional(Positional("pkgid")), Ok("new".to_owned()));
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
        opts.insert("--path".to_owned(), ParamArg(0, vec![Some(Param::Indirect(0))]));
        opts.insert("--verbose".to_owned(), ParamArg(2, vec![Some(Param::Direct("2".to_owned()))]));
        
        let cli = Cli::new(args);
        assert_eq!(cli, Cli {
            positionals: vec![
                Some(PosArg(1, "C:/Users/chase/hdl".to_owned())),
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
        assert_eq!(cli.next_positional(Positional("subcommand")), Ok("build".to_owned()));
        assert_eq!(cli.next_positional(Positional("plugin")), Ok("quartus".to_owned()));
        // verify there are no more positionals
        assert!(cli.next_positional::<String>(Positional("extra")).is_err());
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
                Some(PosArg(0, "new".to_owned())),
                Some(PosArg(1, "rary.gates".to_owned())),
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
        opts.insert("--path".to_owned(), ParamArg(1, vec![Some(Param::Indirect(1))]));
        opts.insert("--verbose".to_owned(), ParamArg(4, vec![Some(Param::Direct("2".to_owned()))]));
        let mut cli = Cli {
            positionals: vec![
                Some(PosArg(0, "new".to_owned())),
                Some(PosArg(2, "C:/Users/chase".to_owned())),
                Some(PosArg(3, "rary.gates".to_owned())),
            ],
            options: opts,
            remainder: Vec::new(),
            past_opts: false,
        };

        assert_eq!(cli.get_option(Optional("--verbose")), Ok(Some(2)));
        assert_eq!(cli.get_option(Optional("--path")), Ok(Some("C:/Users/chase".to_owned())));
        assert_eq!(cli.next_positional(Positional("subcommand")), Ok("new".to_owned()));
        assert_eq!(cli.next_positional(Positional("pkgid")), Ok("rary.gates".to_owned()));

        let args = vec![
            "orbit", "--version", "get", "gates::nor_gate", "--code", "vhdl",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional("--code")), Ok(Some("vhdl".to_string())));
        assert_eq!(cli.next_positional(Positional("subcommand")), Ok("get".to_owned()));
        assert_eq!(cli.next_positional(Positional("pkgid")), Ok("gates::nor_gate".to_owned()));
        assert_eq!(cli.get_flag(Flag("version")), Ok(true));
        assert!(cli.is_clean().is_ok());

        let args = vec![
            "orbit", "--path", "c:/users/chase", "--stats", "info", "--help",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional("--path")), Ok(Some("c:/users/chase".to_string())));
        assert_eq!(cli.next_positional(Positional("subcommand")), Ok("info".to_owned()));
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
            "orbit", "--config", "general.editor=code", "--config", "general.author=chase",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option_vec(Optional("--config")), Ok(Some(vec![
            "general.editor=code".to_string(),
            "general.author=chase".to_string(),
        ])));
    }
}