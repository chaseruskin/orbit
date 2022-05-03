use std::io::Read;

use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::Positional;
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Build {
    command: String,
    args: Vec<String>,
}

impl FromCli for Build {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Build {
            command: cli.require_positional(Positional::new("command"))?,
            args: cli.check_remainder()?,
        });
        command
    }
}

impl Command for Build {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // verify running from an IP directory
        c.goto_ip_path()?;
        // verify a blueprint file exists at build directory
        let blueprint_file = c.get_ip_path().unwrap().join(c.get_build_dir()).join("blueprint.tsv");
        if blueprint_file.exists() == false {
            return Err(Box::new(AnyError(format!("no blueprint file to build from; consider running 'orbit plan'"))))
        }
        // read the .env file
        let env_file = c.get_ip_path().unwrap().join(c.get_build_dir()).join(".env");
        if env_file.exists() == true {
            let mut file = std::fs::File::open(env_file).expect("failed to open .env file");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("failed to read contents");
            // transform into environment variables
            for line in contents.split_terminator('\n') {
                let result = line.split_once('=');
                // set env variables
                if let Some((name, value)) = result {
                    std::env::set_var(name, value);
                }
            }
        }
        self.run()
    }
}

impl Build {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {

        let proc = std::process::Command::new(&self.command)
            .args(&self.args)
            .output()?;
        print!("{}", std::str::from_utf8(&proc.stdout).unwrap());
        if proc.stderr.is_empty() == false {
            return Err(Box::new(AnyError(std::str::from_utf8(&proc.stderr).unwrap().to_string())))
        }
        Ok(())
    }
}

const HELP: &str = "\
Execute a backend tool/workflow.

Usage:
    orbit build [options] <command> [--] [args]...

Args:
    <command>           process to run in orbit

Options:
    -- args...          arguments to pass to the requested command

Use 'orbit help build' to learn more about the command.
";