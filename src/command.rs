use crate::cli;

pub trait Command {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized;

    fn initialize(mut cla: cli::Cli) -> Result<Self, cli::CliError>
    where Self: Sized {
        let cmd = Self::new(&mut cla)?;
        cla.is_clean()?;
        Ok(cmd)
    }
}

// example command demo
#[derive(Debug, PartialEq)]
pub struct Sum {
    digits: Vec<u8>,
    verbose: bool,
}

impl Command for Sum {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        let v = cli::Flag("verbose");

        Ok(Sum { 
            digits: cla.get_option_vec("--digit")?.or(Some(vec![])).unwrap(), 
            verbose: cla.get_flag(v)?,
        })
    }
}

impl Sum {
    pub fn run(self) {
        let mut txt = String::new();
        let s = self.digits.iter().fold(0, |acc, x| {
            txt += &format!("{} + ", x);
            acc + x
        });
        if self.verbose { print!("{}= ", txt.trim_end_matches("+ ")); }
        println!("{}", s);
    }
}

/*
Usage:
    orbit [options] <command> [arguments]

Commands:
    new             create a new orbit ip
    init            create a new orbit ip in an existing directory
    edit            work on an ip in your development path
    install         load a released ip to your orbit cache
    get             add dependencies to current ip
    plan            generate a blueprint file
    build           execute a backend workflow
    launch          release the next version for an ip

Options:
    --version       print the current orbit version
    --help          print help information
    --list          list all plugins and commands

Use "orbit help <command>" for more information about a command.
*/