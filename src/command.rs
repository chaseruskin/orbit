use crate::cli;
use std::fmt::Debug;
use std::str::FromStr;

type Route<T> = Option<T>;
pub type DynCommand = Box<dyn Command>;

pub trait Command: Debug {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized;

    fn initialize(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized {
        // :todo: set the usage before failing and read if --help is there
        let cmd = Self::new(cla)?;
        cla.is_clean()?;
        Ok(cmd)
    }

    // :todo: implement a rules fn to verify all args requested do not conflict
    // example: --lib | --bin, errors if --lib & --bin are passed
    fn verify_rules(&self) -> () { todo!() }

    fn run(&self) -> ();
}

pub trait Dispatch: Debug {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError>;
}

#[derive(Debug)]
enum Subcommand {
    Sum(Route<Sum>),
    NumCast(Route<NumCast>),
}

impl FromStr for Subcommand  {
    type Err = Vec<String>;

    fn from_str(s: & str) -> Result<Subcommand, Self::Err> {
        use Subcommand::*;
        match s {
            "sum" => Ok(Sum(None)),
            "cast" | "c" => Ok(NumCast(None)),
            _ => {
                Err(vec!["sum".to_owned(), "cast".to_owned(), "c".to_owned()])
            }
        }
    }
}

impl Dispatch for Subcommand  {
    fn dispatch(self, cla: &mut cli::Cli) -> Result<DynCommand, cli::CliError> {
        match self {
            Subcommand::Sum(_) => Ok(Box::new(Sum::initialize(cla)?)),
            Subcommand::NumCast(_) => Ok(Box::new(NumCast::initialize(cla)?)),
        }
    }
}

#[derive(Debug)]
pub struct Orbit {
    version: bool,
    help: bool,
    config: Vec<String>,
    color: Option<u8>,
    command: Option<Box<dyn Command>>,
}

impl Command for Orbit {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(Orbit { 
            color: cla.get_option(cli::Optional::new("color"))?,
            config: cla.get_option_vec(cli::Optional::new("config").value("KEY=VALUE"))?.unwrap_or(vec![]),
            help: cla.get_flag(cli::Flag::new("help"))?,
            version: cla.get_flag(cli::Flag::new("version"))?,
            command: cla.next_command::<Subcommand>(cli::Positional::new("subcommand"))?,
        })
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
            println!("orbit is a tool for hdl package management");
            println!("usage:\n\torbit [options] [command]");
        }
    }
}

// example command demo
#[derive(Debug, PartialEq)]
pub struct Sum {
    guess: u8,
    digits: Vec<u8>,
    verbose: bool,
    ver: crate::version::Version,
}

impl Command for Sum {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        let v = cli::Flag::new("verbose");
        Ok(Sum { 
            digits: cla.get_option_vec(cli::Optional::new("digit"))?
                .unwrap_or(vec![]),
            guess: cla.next_positional(cli::Positional::new("guess"))?,
            ver: cla.next_positional(cli::Positional::new("version"))?,
            verbose: cla.get_flag(v)?,
        })
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
            pad: cla.get_option(cli::Optional::new("pad"))?.unwrap_or(0),
            base: cla.get_option_vec(cli::Optional::new("base"))?.unwrap_or(vec![]),
            deci: cla.next_positional(cli::Positional::new("num"))?,
        })
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
