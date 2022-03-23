use crate::cli;
use std::str::FromStr;
use std::fmt::Debug;

pub trait Command: Debug {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized;

    fn initialize(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized {
        // :todo: set the usage before failing
        let cmd = Self::new(cla)?;
        cla.is_clean()?;
        Ok(cmd)
    }

    fn run(&self) -> ();
}

#[derive(Debug, PartialEq)]
enum Subcommand {
    Sum(Sum),
    NumCast(NumCast),
}

#[derive(Debug, PartialEq)]
enum SubcommandError {}

impl FromStr for Subcommand {
    type Err = SubcommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        todo!() 
    }
}

#[derive(Debug)]
pub struct Orbit {
    version: bool,
    help: bool,
    config: Vec<String>,
    command: Box<dyn Command>,
}

impl Command for Orbit {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        // :todo: pass this into next_command?
        let subs = |s: &str| match s {
            "sum" => Some(Subcommand::Sum(Sum::default())),
            "cast"=> Some(Subcommand::NumCast(NumCast::default())),
            _ => None
        };

        Ok(Orbit { 
            config: cla.get_option_vec(cli::Optional("--config"))?.unwrap_or(vec![]),
            help: cla.get_flag(cli::Flag("help"))?,
            version: cla.get_flag(cli::Flag("version"))?,
            command: cla.next_command::<Subcommand>(cli::Positional("subcommand"))?,
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
        }
        self.command.run();
    }
}




// example command demo
#[derive(Debug, PartialEq, Default)]
pub struct Sum {
    guess: u8,
    digits: Vec<u8>,
    verbose: bool,
}

impl Command for Sum {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        let v = cli::Flag("verbose");
        Ok(Sum { 
            digits: cla.get_option_vec(cli::Optional("--digit"))?
                .unwrap_or(vec![]),
            guess: cla.next_positional(cli::Positional("guess"))?,
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
#[derive(Debug, PartialEq, Default)]
pub struct NumCast {
    deci: u32,
    base: u8,
    pad: u8,
}

impl Command for NumCast {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        Ok(NumCast { 
            pad: cla.get_option(cli::Optional("--pad"))?.unwrap_or(0),
            base: cla.get_option(cli::Optional("--base"))?.unwrap_or(10),
            deci: cla.next_positional(cli::Positional("num"))?,
        })
    }

    fn run(&self) {
        let resp = if self.base == 2 {
            format!("{:b}", self.deci)
        } else if self.base == 8 {
            format!("{:o}", self.deci)
        } else if self.base == 16 {
            format!("{:x}", self.deci)
        } else {
            "unsupported base value".to_string()
        };
        println!("{}", resp);
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
