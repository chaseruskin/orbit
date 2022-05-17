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

        // read config.toml for setting any env variables
        if let Some(env_table) = c.get_config().get("env") {
            if let Some(table) = env_table.as_table() {
                let mut table = table.iter();
                while let Some((key, val)) = table.next() {
                    if let Some(val) = val.as_str() {
                        // perform proper env key formatting
                        let key = format!("ORBIT_ENV_{}", key.to_ascii_uppercase().replace('-', "_"));
                        std::env::set_var(key, val);
                    } else {
                        panic!("key 'env.{}' must have string value", key)
                    }
                }
            } else {
                panic!("key 'env' must be a table")
            }
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
        // try to find plugin matching `command` name under the `alias`
        let plug = c.get_plugins().get(&self.command);
        self.run(plug)
    }
}

use crate::core::plugin::Plugin;
use std::process::Stdio;

impl Build {
    fn run(&self, plug: Option<&Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args)
        } else {
            let mut proc = std::process::Command::new(&self.command)
                .args(&self.args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            let exit_code = proc.wait()?;
            match exit_code.code() {
                Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
                None =>  Err(AnyError(format!("terminated by signal")))?
            }
        }
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