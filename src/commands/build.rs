use std::io::Read;

use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Optional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::plugin::Plugin;

#[derive(Debug, PartialEq)]
pub struct Build {
    alias: Option<String>,
    list: bool,
    command: Option<String>,
    args: Vec<String>,
}

impl FromCli for Build {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Build {
            alias: cli.check_option(Optional::new("plugin").value("alias"))?,
            list: cli.check_flag(Flag::new("list"))?,
            command: cli.check_option(Optional::new("command").value("cmd"))?,
            args: cli.check_remainder()?,
        });
        command
        // @TODO remember plugin name as env variable on plan command
        // so it does not have to be re-entered by user (plugin becomes an option
        // on command-line)
    }
}

impl Command for Build {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // display plugin list and exit
        if self.list == true {
            println!("{}", Plugin::list_plugins(&c.get_plugins().values().into_iter().collect::<Vec<&Plugin>>()));
            return Ok(())
        }

        // verify only 1 option is provided
        if self.command.is_some() && self.alias.is_some() {
            return Err(AnyError(format!("cannot execute both a plugin and command")))?
        }
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
        let plug = if let Some(a) = &self.alias {
            match c.get_plugins().get(a) {
                Some(p) => Some(p),
                None => return Err(AnyError(format!("no plugin named '{}'", a)))?,
            }
        } else {
            None
        };

        if plug.is_none() && self.command.is_none() {
            return Err(AnyError(format!("pass a plugin or a command for building")))?
        }

        self.run(plug)
    }
}

use std::process::Stdio;

impl Build {
    fn run(&self, plug: Option<&Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args)
        } else if let Some(cmd) = &self.command {
            let mut proc = std::process::Command::new(cmd)
                .args(&self.args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            let exit_code = proc.wait()?;
            match exit_code.code() {
                Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
                None =>  Err(AnyError(format!("terminated by signal")))?
            }
        } else {
            Ok(())
        }
    }
}

const HELP: &str = "\
Execute a backend tool/workflow.

Usage:
    orbit build [options] [--] [args]...

Options:
    --plugin <alias>    plugin to execute
    --command <cmd>     command to execute
    --list              view available plugins
    -- args...          arguments to pass to the requested command

Use 'orbit help build' to learn more about the command.
";