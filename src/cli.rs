//! File     : cli.rs
//! Abstract :
//!     The command-line interface parses user's requests into program code.
//! 
//! Notes    :
//! - options must be queried before positionals
//! - parameters must be provided only after their respective subcommands
//! Todo    :
//! - allow lowercase option lookups
//! - allow shorthand options with single dash
//! - use `known_args` to compute edit distance among options and report a close find

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
    known_args: Vec<Arg>,
}

#[derive(Debug, PartialEq)]
enum Param {
    Direct(String),
    Indirect(usize),
}

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
            known_args: Vec::new(),
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

    /// Pops off the next positional in the provided order.
    pub fn next_positional<T: FromStr + std::fmt::Debug>(&mut self, arg: Positional) -> Result<T, CliError>
        where <T as std::str::FromStr>::Err: std::error::Error {
        self.past_opts = true;
        if let Some(p) = self.positionals.iter_mut()
            .skip_while(|s| s.is_none())
            .next() {
                match p.take().unwrap().1.parse::<T>() {
                    Ok(r) => {
                        self.known_args.push(Arg::Positional(arg));
                        Ok(r) 
                    }
                    Err(e) => Err(CliError::BadType(Arg::Positional(arg), format!("{}", e)))
                }
        } else {
            // found zero available arguments
            Err(CliError::MissingPositional(Arg::Positional(arg)))
        }
    }

    /// Queries if a flag was raised once. 
    /// 
    /// __Errors__: if a direct value was given or if the flag was raised multiple times
    pub fn get_flag(&mut self, flag: Flag) -> Result<bool, CliError> {
        // // check if it is in the map
        let raised = if let Some(mut val) = self.options.remove(&flag.to_string()) {
            // raise error if there is an attached option to the flag
            if val.1.len() > 1 {
                // err: duplicate values
                return Err(CliError::DuplicateOptions(Arg::Flag(flag)))
            } else {
                match val.1.pop().unwrap() {
                    Some(p) => match p {
                        Param::Direct(s) => return Err(CliError::UnexpectedValue(Arg::Flag(flag), s)),
                        Param::Indirect(_) => true,
                    }
                    None => true,
                }
            }
        // flag was not found in options map
        } else {
            false
        };
        self.known_args.push(Arg::Flag(flag));
        Ok(raised)
    }

    /// Filters out undetermined options that have a position < i.
    pub fn is_partial_clean(&self, i: usize) -> Result<(), CliError> {
        if let Some(arg) = self.options
            .iter()
            .find(|(_, o)| { o.0 <= i }) {
                Err(CliError::OutOfContextArg(arg.0.to_string()))
        } else {
            Ok(())
        }
    }

    /// Checks that there are no unused/unchecked arguments.
    pub fn is_clean(&self) -> Result<(), CliError> {
        // errors if `options` is not empty or `positionals` has a non-None value.
        if self.options.is_empty() != true {
            // :todo: compute edit distance on known args to generate suggestion
            let unknown = self.options.keys().next().unwrap().to_owned();
            Err(CliError::UnexpectedArg(unknown))
        } else if let Some(Some(unknown)) = self.positionals.iter().find(|f| f.is_some()) {
            Err(CliError::UnexpectedArg(unknown.1.to_owned())) 
        } else {
            Ok(())
        }
    }

    /// Retuns the vector of leftover arguments split by '--' and removes that flag from the `options` map.
    pub fn get_remainder(&mut self) -> &Vec::<String> {
        self.options.remove("--");
        return &self.remainder
    }

    /// Queries for a particular option to get it's value.
    /// 
    /// To set a default value, chain `.unwrap_or()` to this function call.
    pub fn get_option<T: FromStr + std::fmt::Debug>(&mut self, opt: Optional) -> Result<Option<T>, CliError>
    where <T as std::str::FromStr>::Err: std::error::Error {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        // check if it is in the map (pull from map)
        let o = if let Some(mut m) = self.options.remove(&opt.get_flag().to_string()) { 
            // add optional to the known args
            if m.1.len() > 1 {
                return Err(CliError::DuplicateOptions(Arg::Optional(opt)));
            // investigate if the user provided a param for the option
            } else if let Some(p) = m.1.pop().unwrap() {
                Some(self.parse_param(p, &opt)?)
            } else {
                return Err(CliError::ExpectingValue(Arg::Optional(opt)));
            }
        } else { 
            None
        };
        self.known_args.push(Arg::Optional(opt));
        Ok(o)
    }

    /// Queries for a particular option and returns all supplied values.
    /// 
    /// To set a default value, chain `.unwrap_or()` to this function call.
    pub fn get_option_vec<T: FromStr + std::fmt::Debug>(&mut self, opt: Optional) -> Result<Option<Vec<T>>, CliError>
    where <T as std::str::FromStr>::Err: std::error::Error {
        if self.past_opts { 
            panic!("options must be evaluated before positionals")
        }
        if let Some(m) = self.options.remove(&opt.get_flag().to_string()) {
            let mut res = Vec::<T>::with_capacity(m.1.len());
            let mut m = m.1.into_iter();
            while let Some(e) = m.next() {
                if let Some(p) = e {
                    res.push(self.parse_param(p, &opt)?);
                } else {
                    return Err(CliError::ExpectingValue(Arg::Optional(opt)));
                }
            }
            self.known_args.push(Arg::Optional(opt));
            Ok(Some(res))
        // option was not provided by user, return None
        } else {
            Ok(None)
        }
    }

    /// Handles updating the positional vector depending on if a paramater was direct or indirect.
    fn parse_param<T: FromStr + std::fmt::Debug>(&mut self, p: Param, opt: &Optional) -> Result<T, CliError>
    where <T as std::str::FromStr>::Err: std::error::Error {
        match p {
            Param::Direct(s) => {
                match s.parse::<T>() {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CliError::BadType(Arg::Optional(opt.clone()), format!("{}", e)))
                }
            }
            Param::Indirect(i) => {
                // perform a swap on the data unless it has already been used up
                match self.positionals[i].take().expect("value was stolen by positional").1.parse::<T>() {
                    Ok(r) => Ok(r),
                    Err(e) => Err(CliError::BadType(Arg::Optional(opt.clone()), format!("{}", e)))
                }
            }
        }
   }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Positional {
    name: String
}

impl Positional {
    pub fn new(s: &str) -> Self {
        Positional { name: s.to_string() }
    }
}

impl Display for Positional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "<{}>", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Flag {
    name: String
}

impl Flag {
    pub fn new(s: &str) -> Self {
        Flag { name: s.to_string() }
    }
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "--{}", self.name)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Optional {
    name: Flag,
    value: Positional,
}

impl Optional {
    pub fn new(s: &str) -> Self {
        Optional { 
            name: Flag::new(s),
            value: Positional::new(s),
        }
    }
    
    pub fn get_flag(&self) -> &Flag {
        &self.name
    }

    pub fn value(mut self, s: &str) -> Self {
        self.value.name = s.to_string();
        self
    }
}

impl Display for Optional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{} {}", self.name, self.value)
    }
}

#[derive(Debug, PartialEq)]
pub enum Arg {
    Positional(Positional),
    Flag(Flag),
    Optional(Optional),
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::Flag(a) => write!(f, "{}", a),
            Self::Positional(a) => write!(f, "{}", a),
            Self::Optional(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    BadType(Arg, String),
    MissingPositional(Arg),
    DuplicateOptions(Arg),
    ExpectingValue(Arg),
    UnexpectedValue(Arg, String),
    OutOfContextArg(String),
    UnexpectedArg(String),
}

impl Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use CliError::*;
        match self {
            OutOfContextArg(o) => write!(f, "invalid argument in current context '{}'", o),
            BadType(a, e) => write!(f, "argument '{}' has {}", a, e),
            MissingPositional(p) => write!(f, "missing positional '{}'", p),
            DuplicateOptions(o) => write!(f, "option '{}' was requested more than once, but can only be supplied once", o),
            ExpectingValue(x) => write!(f, "option '{}' expects a value but none was supplied", x),
            UnexpectedValue(x, s) => write!(f, "flag '{}' cannot accept values but one was supplied \"{}\"", x, s),
            UnexpectedArg(s) => write!(f, "unknown argument '{}'", s),
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
            known_args: Vec::new(),
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
            known_args: Vec::new(),
        });
    }

    #[test]
    #[should_panic = "options must be evaluated before positionals"]
    fn options_after_positionals() {
        let args = vec![
            "orbit", "--config", "general:editor=code", "new", "rary.gates", "--open", "--lib"
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_flag(Flag::new("lib")), Ok(true));
        assert_eq!(cli.get_option::<String>(Optional::new("log")), Ok(None));
        // undesired here... options must be evaluated before positionals
        assert_eq!(cli.next_positional(Positional::new("subcommand")), Ok("general:editor=code".to_owned()));
        assert_eq!(cli.next_positional(Positional::new("pkgid")), Ok("new".to_owned()));
        // panic occurs on first call to an option once beginning to read positionals
        assert_eq!(cli.get_option(Optional::new("config")), Ok(Some("general:editor=code".to_owned())));
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
            known_args: Vec::new(),
        });
    }

    #[test]
    fn passing_args_with_double_dash() {
        let args = vec![
            "orbit", "build", "quartus", "--plan", "--", "synthesize", "--log", "./quartus.log"
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option::<String>(Optional::new("path")), Ok(None));
        assert_eq!(cli.get_option::<String>(Optional::new("log")), Ok(None));
        assert_eq!(cli.next_positional(Positional::new("subcommand")), Ok("build".to_owned()));
        assert_eq!(cli.next_positional(Positional::new("plugin")), Ok("quartus".to_owned()));
        // verify there are no more positionals
        assert!(cli.next_positional::<String>(Positional::new("extra")).is_err());
        assert_eq!(cli.get_flag(Flag::new("plan")), Ok(true));
        assert_eq!(cli.get_flag(Flag::new("version")), Ok(false));
        assert_eq!(cli.get_flag(Flag::new("help")), Ok(false));
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
            known_args: Vec::new(),
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
            known_args: Vec::new(),
        };

        assert_eq!(cli.get_option(Optional::new("verbose")), Ok(Some(2)));
        assert_eq!(cli.get_option(Optional::new("path")), Ok(Some("C:/Users/chase".to_owned())));
        assert_eq!(cli.next_positional(Positional::new("subcommand")), Ok("new".to_owned()));
        assert_eq!(cli.next_positional(Positional::new("pkgid")), Ok("rary.gates".to_owned()));

        let args = vec![
            "orbit", "--version", "get", "gates::nor_gate", "--code", "vhdl",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional::new("code")), Ok(Some("vhdl".to_string())));
        assert_eq!(cli.next_positional(Positional::new("subcommand")), Ok("get".to_owned()));
        assert_eq!(cli.next_positional(Positional::new("pkgid")), Ok("gates::nor_gate".to_owned()));
        assert_eq!(cli.get_flag(Flag::new("version")), Ok(true));
        assert!(cli.is_clean().is_ok());

        let args = vec![
            "orbit", "--path", "c:/users/chase", "--stats", "info", "--help",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option(Optional::new("path")), Ok(Some("c:/users/chase".to_string())));
        assert_eq!(cli.next_positional(Positional::new("subcommand")), Ok("info".to_owned()));
        assert_eq!(cli.get_flag(Flag::new("stats")), Ok(true));
        assert_eq!(cli.get_flag(Flag::new("version")), Ok(false));
        assert_eq!(cli.get_flag(Flag::new("help")), Ok(true));
        assert!(cli.is_clean().is_ok());
    }

    #[test]
    fn unknown_args() {
        let args = vec![
            "orbit", "--version", "--unknown",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);
        // command only expects these two flags
        assert_eq!(cli.get_flag(Flag::new("version")), Ok(true));
        assert_eq!(cli.get_flag(Flag::new("help")), Ok(false));
        // --unknown was not caught
        assert!(cli.is_clean().is_err());
    }

    #[test]
    fn dupe_options() {
        let args = vec![
            "orbit", "--config", "general.editor=code", "--config", "general.author=chase",
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        assert_eq!(cli.get_option_vec(Optional::new("config")), Ok(Some(vec![
            "general.editor=code".to_string(),
            "general.author=chase".to_string(),
        ])));
    }

    #[test]
    fn known_args() {
        let args = vec![
            "orbit", 
            "--config", "key1=value1", 
            "--config=key2=value2",
            "--verbose",
            "new",
            "--f1",
            "--f2",
            "--o1=10"
        ].into_iter().map(|s| s.to_owned());
        let mut cli = Cli::new(args);

        let cfg = Optional::new("config").value("KEY=VALUE");
        let verbose = Flag::new("verbose");
        let quiet = Flag::new("quiet");
        let sub = Positional::new("subcommand");
        let f1 = Flag::new("f1");
        let f2 = Flag::new("f2");
        let f3 = Flag::new("f3");
        let o1 = Optional::new("o1").value("NUM");

        assert_eq!(cli.get_option_vec(cfg.clone()).unwrap(), Some(vec![
            "key1=value1".to_string(),
            "key2=value2".to_string(),
        ]));
        assert_eq!(cli.get_option(o1.clone()).unwrap(), Some(10));
        assert_eq!(cli.get_flag(verbose.clone()).unwrap(), true);
        assert_eq!(cli.get_flag(quiet.clone()).unwrap(), false);
        assert_eq!(cli.next_positional::<String>(sub.clone()).unwrap(), "new");
        assert_eq!(cli.get_flag(f1.clone()).unwrap(), true);
        assert_eq!(cli.get_flag(f2.clone()).unwrap(), true);
        assert_eq!(cli.get_flag(f3.clone()).unwrap(), false);

        assert_eq!(cli.known_args, vec![
            Arg::Optional(cfg), Arg::Optional(o1), Arg::Flag(verbose),
            Arg::Flag(quiet), Arg::Positional(sub), 
            Arg::Flag(f1), Arg::Flag(f2), Arg::Flag(f3),
        ]);
    }
}