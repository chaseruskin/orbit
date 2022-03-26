use crate::cli::{self, CliError};
use std::fmt::Debug;
use std::str::FromStr;
use std::error::Error;
use crate::arg::*;

pub type DynCommand = Box<dyn Command>;

pub trait Command: Debug {
    /// Pulls data from the command-line args into the `Command` struct.
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> where Self: Sized;

    /// Performs various checks and calls `new` to generate a struct implementing the `Command` trait.
    fn load(cla: &mut cli::Cli) -> Result<Self, cli::CliError> where Self: Sized {
        cla.set_usage(&Self::usage());
        let cmd = match Self::new(cla) {
            Ok(c) => {
                if cla.asking_for_help() {
                    println!("{}", Self::help());
                    std::process::exit(0);
                }
                c
            },
            Err(e) => {
                if let cli::CliError::SuggestArg(..) = e {
                    return Err(e)
                } else {
                    // check if help is asked for because we errored
                    if cla.asking_for_help() {
                        println!("{}", Self::help());
                        std::process::exit(0);
                    } else if let cli::CliError::MissingPositional(..) = e {
                        cla.is_clean()?;
                    }
                    return Err(e);
                }
            }
        };
        cla.is_clean()?;
        cla.has_no_extra_positionals()?;
        cmd.verify_rules()?;
        Ok(cmd)
    }

    /// Returns general command usage syntax.
    fn usage() -> String where Self: Sized;

    /// Returns short command help guide.
    fn help() -> String where Self: Sized;

    /// Validates that arguments are logically grouped on the command-line.
    fn verify_rules(&self) -> Result<(), cli::CliError> { 
        Ok(())
    }

    /// Executes the command.
    fn run(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Dispatch: Debug {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError>;
}

#[derive(Debug)]
enum Subcommand {
    Sum,
    NumCast,
    Help,
}

impl FromStr for Subcommand  {
    type Err = Vec<String>;

    fn from_str(s: & str) -> Result<Self, Self::Err> {
        Ok(match s {
            "sum" => Subcommand::Sum,
            "cast" => Subcommand::NumCast,
            "help" => Subcommand::Help,
            _ => return Err(vec![
                "sum".to_owned(), 
                "cast".to_owned(), 
                "help".to_owned()
                ])
        })
    }
}

impl Dispatch for Subcommand  {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError> {
        match self {
            Subcommand::Sum => Ok(Box::new(Sum::load(cla)?)),
            Subcommand::NumCast => Ok(Box::new(NumCast::load(cla)?)),
            Subcommand::Help => Ok(Box::new(Help::load(cla)?)),
        }
    }
}

#[derive(Debug)]
pub struct Orbit {
    version: bool,
    help: bool,
    config: Vec<String>,
    upgrade: bool,
    color: Option<u8>,
    command: Option<DynCommand>,
}

impl Command for Orbit {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(Orbit { 
            help   : cla.get_flag(Flag::new("help").short('h'))?,
            version: cla.get_flag(Flag::new("version"))?,
            upgrade: cla.get_flag(Flag::new("upgrade"))?,
            color  : cla.get_option(Optional::new("color"))?,
            config : cla.get_option_vec(Optional::new("config").value("KEY=VALUE").short('c'))?.unwrap_or(vec![]),
            command: cla.next_command::<Subcommand>(Positional::new("subcommand"))?,
        })
    }

    fn run(&self) -> Result <(), Box<dyn Error>> {
        self.config.iter().for_each(|f| {
            if let Some((k, v)) = f.split_once("=") {
                println!("key: {}\tvalue: {}", k, v);
            }
        });
        if self.version {
            println!("orbit 0.1.0");
        } else if let Some(cmd) = &self.command {
            cmd.run()?;
        } else {
            println!("{}", Self::help())
        }
        Ok(())
    }

    fn usage() -> String {
        format!("\nUsage:\n    orbit [options] <command>")
    }

    fn help() -> String {
        format!("{}", OVERVIEW)
    }
}

// example command demo
#[derive(Debug, PartialEq)]
pub struct Sum {
    guess: u8,
    digits: Vec<u8>,
    verbose: bool,
    pkg: crate::pkgid::PkgId,
}

impl Command for Sum {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(Sum { 
            verbose: cla.get_flag(Flag::new("verbose"))?,
            digits: cla.get_option_vec(Optional::new("digit").value("N").short('d'))?
                .unwrap_or(vec![]),
            guess: cla.next_positional(Positional::new("guess"))?,
            pkg: cla.next_positional(Positional::new("pkgid"))?,
        })
    }

    fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut txt = String::new();
        let s = self.digits.iter().fold(0, |acc, x| {
            txt += &format!("{} + ", x);
            acc + x
        });
        if self.verbose { print!("{}= ", txt.trim_end_matches("+ ")); }
        println!("{}", s);
        if s == self.guess {
            println!("you guessed right!");
        } else {
            println!("you guessed incorrectly.");
        }
        Ok(())
    }

    fn verify_rules(&self) -> Result<(), cli::CliError> {
        if let Err(e) = self.pkg.fully_qualified() {
            return Err(cli::CliError::BadType(Arg::Positional(Positional::new("pkgid")), e.to_string()));
        }
        Ok(())
    }

    fn usage() -> String {
        format!("\nUsage:\n    orbit sum [options] <guess> <pkgid>")
    }

    fn help() -> String {
format!("Add multiple numbers together
{}

Options:
    --verbose           print out the math equation
    --digit, -d <N>...  give a digit to include in the summation

Args:
    <guess>         a number to compare against the summation
    <pkgid>         a fully qualified pkgid

Run 'orbit help sum' for more details.
", Self::usage())
    }
}

// example command demo
#[derive(Debug, PartialEq)]
pub struct NumCast {
    deci: u32,
    base: Vec<u8>,
    pad: u8,
}

impl Command for NumCast {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(NumCast { 
            pad: cla.get_option(Optional::new("pad"))?.unwrap_or(0),
            base: cla.get_option_vec(Optional::new("base"))?.unwrap_or(vec![]),
            deci: cla.next_positional(Positional::new("num"))?,
        })
    }

    fn run(&self) -> Result<(), Box<dyn Error>>{
        self.base.iter().for_each(|&b| {
            match b {
                2 => println!("0b{:b}", self.deci),
                8 => println!("0o{:o}", self.deci),
                10 => println!("{}", self.deci),
                16 => println!("0x{:x}", self.deci),
                _ => println!("error: base {} is unsupported", b),
            }
        });
        Ok(())
    }

    fn usage() -> String {
        format!("\nUsage:\n    orbit cast [options] <num>")
    }

    fn help() -> String {
format!("Convert a decimal number to a different base
{}

Options:
    --base <N>...   numbering system to convert to [2, 8, 10, 16]
    --pad  <N>      number of leading zeros

Args:
    <num>           a decimal number

Run 'orbit help sum' for more details.
", Self::usage())
    }
}

#[derive(Debug, PartialEq)]
enum Topic {
    Cast,
    Sum,
    Help,
}

impl FromStr for Topic {
    type Err = cli::CliError;

    fn from_str(s: & str) -> Result<Self, Self::Err> {
        use Topic::*;
        Ok(match s {
            "sum" => Sum,
            "cast" => Cast,
            _ => return Err(CliError::BrokenRule(s.to_owned()))
        })
    }
}

// example command demo
#[derive(Debug)]
pub struct Help {
    topic: Topic,
}

impl Command for Help {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        let topic = match cla.next_positional(Positional::new("command")) {
            // default the topic to be in-depth about orbit if none was supplied
            Err(cli::CliError::MissingPositional(..)) => {
                Ok(Topic::Help)
            }
            // try to suggest a topic if none was provided
            Err(cli::CliError::BadType(_, s)) => {
                match crate::seqalin::sel_min_edit_str(&s, &vec!["sum".to_owned(), "cast".to_owned()], 4) {
                    Some(w) => Err(cli::CliError::SuggestArg(s.to_owned(), w.to_owned())),
                    _ => Err(cli::CliError::UnknownSubcommand(Arg::Positional(Positional::new("command")), s.to_owned()))
                }
            }
            Err(e) => Err(e),
            Ok(o) => Ok(o),
        };
        Ok(Help { 
            topic: topic?,
        })
    }

    fn run(&self) -> Result<(), Box<dyn Error>> { 
        match &self.topic {
            Topic::Help => println!("{}", "in-depth about orbit"),
            Topic::Cast => println!("{}", "more help for cast"),
            Topic::Sum => println!("{}", "more help for sum"),
        }
        Ok(())
    }

    fn usage() -> String { 
        format!("\nUsage:\n    orbit help <command>")
    }

    fn help() -> String { 
        format!("{}", OVERVIEW)
    }
}
    

/*
Orbit is a tool for hdl package management.

Usage:
    orbit <command> [arguments]

Commands:
    new             create a new orbit ip
    init            create a new orbit ip in an existing directory
    edit            work on an ip in your development path
    install         load a released ip to your orbit cache
    get             add dependencies to current ip
    plan            generate a blueprint file
    build           execute a backend workflow
    launch          release the next version for an ip
    list            list all plugins and command

Use "orbit help <command>" for more information about a command.
*/

/*
Create a new orbit ip as <pkgid>

Usage:
    orbit new [options] <pkgid> 

Options:
    --path <PATH>       destination to create ip (default is ORBIT_WORKSPACE)
    --template <PATH>   a directory to be used as a template
    --vcs <VCS>         initialize a new repository (git) or none (none)

Args:
    <pkgid>             a fully-qualified pkgid
*/

const OVERVIEW: &str = "orbit is a tool for hdl package management.

Usage:
    orbit [options] <command>

Options:
    --config, -c <KEY=VALUE>    override a configuration settings
    --color <INT>               set the color intensity
    --upgrade                   check for the latest orbit binary
    --version                   print the version and exit
    --help, -h                  print help information

Commands:
    cast            convert a decimal number to a different base [test]
    sum             add up a variable amount of numbers [test]

Use 'orbit help <command>' for more information about a command.
";