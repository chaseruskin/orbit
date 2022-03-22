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
    guess: u8,
    digits: Vec<u8>,
    verbose: bool,
}

impl Command for Sum {
    fn new(cla: &mut cli::Cli) -> Result<Self, cli::CliError> {
        let v = cli::Flag("verbose");

        Ok(Sum { 
            digits: cla.get_option_vec("--digit")?.or(Some(vec![])).unwrap(),
            guess: cla.next_positional()?,
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
        if s == self.guess {
            println!("you guessed right!");
        } else {
            println!("you guessed incorrectly.");
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
