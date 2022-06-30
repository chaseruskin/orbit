use colored::*;
use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Config {
    global: bool,
    local: bool,
    append: Vec<String>,
    set: Vec<String>,
    unset: Vec<String>,
}

impl FromCli for Config {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Config {
            global: cli.check_flag(Flag::new("global"))?,
            local: cli.check_flag(Flag::new("local"))?,
            append: cli.check_option_all(Optional::new("append"))?.unwrap_or(Vec::new()),
            set: cli.check_option_all(Optional::new("set"))?.unwrap_or(Vec::new()),
            unset: cli.check_option_all(Optional::new("unset"))?.unwrap_or(Vec::new()),
        });
        command
    }
}

use crate::core::config;

impl Command for Config {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // check if we are using global or local
        if self.local && self.global {
            return Err(AnyError(format!("'{}' and '{}' cannot be set at the same time", "--local".yellow(), "--global".yellow())))?
        }
        let mut cfg = if self.local == true {
            match c.get_ip_path() {
                Some(path) => config::Config::from_path(&path.join(".orbit").join(config::CONFIG_FILE))?,
                None => return Err(AnyError(format!("no ip detected in the current directory to modify local configurations")))?,
            }
        } else {
            // duplicate the configuration so we can potentially mutate it
            config::Config::from_path(&c.get_config().get_root().join(config::CONFIG_FILE)).expect("already should be parsed correctly")
        };
        // modify the settings for cfg file
        self.run(&mut cfg)
    }
}

impl Config {
    fn run(&self, cfg: &mut config::Config) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Modify configuration values.

Usage:
    orbit config [options]
    
Options:
    --global                    access the home configuration file
    --local                     access the current project configuration file
    --append <key>=<value>...   add a value to a key storing a list
    --set <key>=<value>...      write the value at the key entry
    --unset <key>...            delete the key's entry

Use 'orbit help config' to learn more about the command.
";