use orbit::interface::cli::*;
use orbit::interface::errors::*;
use orbit::interface::command::*;
use orbit::interface::arg::*;

fn main() {
    let mut cli = Cli::tokenize(std::env::args());
    match Add::from_cli(&mut cli) {
        Ok(r) => {
            std::mem::drop(cli);
            r.exec()
        },
        Err(e) => {
            match e {
                CliError::Help(s) => println!("{}", s),
                _ => eprintln!("error: {}", e),
            }
        }
    }
}

// demo program
#[derive(Debug, PartialEq)]
struct Add {
    lhs: u32,
    rhs: u32,
    verbose: bool,
    help: bool,
    rem: Vec<String>,
}

impl Command for Add {
    fn exec(&self) -> () {
        println!("{}", self.run());
    }
}

impl Add {
    /// Simple fn to return an answer for the `Add` test command.
    fn run(&self) -> String {
        let sum = self.lhs + self.rhs;
        match self.verbose {
            true => format!("{} + {} = {}", self.lhs, self.rhs, sum),
            false => format!("{}", sum),
        }
    }
}

impl FromCli for Add {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let m = Ok(Add {
            help: cli.check_flag(Flag::new("help"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            lhs: cli.require_positional(Positional::new("lhs"))?,
            rhs: cli.require_positional(Positional::new("rhs"))?,
            rem: cli.check_remainder()?,
        });
        cli.is_empty()?;
        m
    }
}

const HELP: &str = "\
a demo program utilizing the cli for orbit

Usage:
    add [options] <lhs> <rhs>

Args:
    <lhs>   a numeric value
    <rhs>   a numeric value

Options:
    --verbose   display detailed computation

Use 'orbit help <command>' for more information about a command.
";