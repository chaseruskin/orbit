use crate::cli;
use std::fmt::Debug;
use std::process::exit;
use std::str::FromStr;
use crate::arg::*;

type Route<T> = Option<T>;
pub type DynCommand = Box<dyn Command>;

pub trait Command: Debug {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized;

    fn initialize(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized {
        let cmd = match Self::new(cla) {
            Ok(c) => {
                match cla.asking_for_help() {
                    true => {
                        Self::route_help(cla);
                        exit(0);
                    }
                    false => c,
                }
            },
            Err(e) => {
                match e {
                    cli::CliError::SuggestArg(..) => {
                        return Err(e);
                    }
                    _ => {
                        // check if help is asked for because we errored
                        match cla.asking_for_help() {
                            true => {
                                Self::route_help(cla);
                                exit(0);
                            }
                            false => return Err(e)
                        }
                    }
                }

            }
        };
        cla.is_clean()?;
        cmd.verify_rules()?;
        Ok(cmd)
    }

    fn route_help(cla: &mut cli::Cli) -> () where Self: Sized {
        match cla.next_command::<Subcommand>(Positional::new("subcommand")) {
            Ok(None) | Err(_) => println!("{}", Self::help()),
            //Err(e) => println!("{}", e),
            _ => ()
        }
    }

    fn usage(&self) -> &str;

    fn help() -> String where Self: Sized;

    // :todo: implement a rules fn to verify all args requested do not conflict
    // example: --lib | --bin, errors if --lib & --bin are passed
    fn verify_rules(&self) -> Result<(), cli::CliError> { 
        Ok(())
    }

    fn run(&self);
}

pub trait Branch: Debug {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError>;
}

#[derive(Debug)]
enum Subcommand {
    Sum(Route<Sum>),
    NumCast(Route<NumCast>),
    Help(Route<Help>),
}

impl FromStr for Subcommand  {
    type Err = Vec<String>;

    fn from_str(s: & str) -> Result<Subcommand, Self::Err> {
        use Subcommand::*;
        match s {
            "sum" => Ok(Sum(None)),
            "cast" => Ok(NumCast(None)),
            "help" => Ok(Help(None)),
            _ => {
                Err(vec!["sum".to_owned(), "cast".to_owned(), "help".to_owned()])
            }
        }
    }
}

impl Branch for Subcommand  {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError> {
        match self {
            Subcommand::Sum(_) => Ok(Box::new(Sum::initialize(cla)?)),
            Subcommand::NumCast(_) => Ok(Box::new(NumCast::initialize(cla)?)),
            Subcommand::Help(_) => Ok(Box::new(Help::initialize(cla)?)),
        }
    }
}

#[derive(Debug)]
pub struct Orbit {
    version: bool,
    help: bool,
    config: Vec<String>,
    color: Option<u8>,
    command: Option<DynCommand>,
}

impl Command for Orbit {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        // :idea: allow cli to first take in data about argument names, then suggest if a mismatch exists
        Ok(Orbit { 
            help : cla.get_flag(Flag::new("help"))?,
            color: cla.get_option(Optional::new("color"))?,
            config: cla.get_option_vec(Optional::new("config").value("KEY=VALUE"))?.unwrap_or(vec![]),
            version: cla.get_flag(Flag::new("version"))?,
            command: cla.next_command::<Subcommand>(Positional::new("subcommand"))?,
        })
    }

    fn help() -> String {
"orbit is a tool for hdl package management.

Usage:
    orbit [options] <command>

Commands:
    cast            convert a decimal number to a different base [test]
    sum             add up a variable amount of numbers [test]

Options:
    --config <KEY=VALUE>    override a configuration settings
    --color <INT>           set the color intensity
    --version               print the version and exit
    --help                  print help information

Use 'orbit help <command>' for more information about a command.
".to_string()
    }

    fn usage(&self) -> &str {
        "orbit [options] <subcommand>"
    }

    fn run(&self) {
        self.config.iter().for_each(|f| {
            if let Some((k, v)) = f.split_once("=") {
                println!("key: {}\tvalue: {}", k, v);
            }
        });
        if self.version {
            println!("orbit 0.1.0");
        } else if let Some(cmd) = &self.command {
            cmd.run();
        } else {
            println!("{}", Self::help())
        }
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
        let v = Flag::new("verbose");
        Ok(Sum { 
            digits: cla.get_option_vec(Optional::new("digit").value("N"))?
                .unwrap_or(vec![]),
            guess: cla.next_positional(Positional::new("guess"))?,
            pkg: cla.next_positional(Positional::new("pkgid"))?,
            verbose: cla.get_flag(v)?,
        })
    }

    fn usage(&self) -> &str {
        "orbit sum [options] <guess> <pkgid>"
    }

    fn help() -> String {
"Add multiple numbers together

Usage:
    orbit sum [options] <guess> <pkgid>

Args:
    <guess>         a number to compare against the summation
    <pkgid>         a fully qualified pkgid

Options:
    --verbose       print out the math equation
    --digit <N>...  give a digit to include in the summation

Run 'orbit help sum' for more details.
".to_string()
    }

    fn verify_rules(&self) -> Result<(), cli::CliError> {
        if let Err(e) = self.pkg.fully_qualified() {
            return Err(cli::CliError::BrokenRule(format!("<pkgid> is {}", e.to_string())));
        }
        Ok(())
    }

    fn run(&self) {
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

    fn usage(&self) -> &str {
        "orbit cast [options] <num>"
    }

    fn help() -> String {
"Convert a decimal number to a different base

Usage:
    orbit cast [options] <num>

Args:
    <num>           a decimal number

Options:
    --base <N>...   numbering system to convert to [2, 8, 10, 16]
    --pad <N>       number of leading zeros

Run 'orbit help sum' for more details.
".to_string()
    }

    fn run(&self) {
        self.base.iter().for_each(|&b| {
            match b {
                2 => println!("0b{:b}", self.deci),
                8 => println!("0o{:o}", self.deci),
                10 => println!("{}", self.deci),
                16 => println!("0x{:x}", self.deci),
                _ => println!("error: base {} is unsupported", b),
            }
        });
    }
}

#[derive(Debug, PartialEq)]
enum Topic {
    Cast,
    Sum,
}

impl FromStr for Topic {
    type Err = Vec<String>;

    fn from_str(s: & str) -> Result<Topic, Self::Err> {
        use Topic::*;
        match s {
            "sum" => Ok(Sum),
            "cast" => Ok(Cast),
            _ => {
                Err(vec!["sum".to_owned(), "cast".to_owned()])
            }
        }
    } 
}

// example command demo
#[derive(Debug)]
pub struct Help {
    topic: Option<DynCommand>,
}

impl Command for Help {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(Help { 
            topic: cla.next_command::<Subcommand>(Positional::new("<command>"))?,
        })
    }

    fn usage(&self) -> &str { 
        todo!() 
    }

    fn help() -> String { 
"orbit is a tool for hdl package management.

Usage:
    orbit [options] <command>

Commands:
    cast            convert a decimal number to a different base [test]
    sum             add up a variable amount of numbers [test]

Options:
    --config <KEY=VALUE>    override a configuration settings
    --color <INT>           set the color intensity
    --version               print the version and exit
    --help                  print help information

Use 'orbit help <command>' for more information about a command.
".to_string()
    }

    fn run(&self) { 
        match &self.topic {
            Some(t) => println!("{:?}", t),
            None => println!("{}", Self::help()),
        }
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
