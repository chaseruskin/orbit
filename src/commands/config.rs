use std::str::FromStr;

use colored::*;
use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Flag, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::core::v2::manifest::FromFile;
use crate::util::anyerror::AnyError;
use crate::OrbitResult;


#[derive(Debug, PartialEq)]
pub struct Entry(String, String);

impl FromStr for Entry {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split on first '=' sign
        match s.split_once('=') {
            Some(e) => Ok(Entry(e.0.to_owned(), e.1.to_owned())),
            None => Err(AnyError(format!("missing '=' separator")))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Config {
    global: bool,
    local: bool,
    append: Vec<Entry>,
    set: Vec<Entry>,
    unset: Vec<String>,
}

impl FromCli for Config {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
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

use crate::core::v2::config;

impl Command<Context> for Config {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // check if we are using global or local
        if self.local && self.global {
            return Err(AnyError(format!("'{}' and '{}' cannot be set at the same time", "--local".yellow(), "--global".yellow())))?
        }
        let mut cfg = if self.local == true {
            match c.get_ip_path() {
                Some(path) => config::Config::from_file(&path.join(".orbit").join(config::CONFIG_FILE))?,
                None => return Err(AnyError(format!("no ip detected in the current directory to modify local configurations")))?,
            }
        } else {
            // duplicate the configuration so we can potentially mutate it
            config::Config::from_file(&c.get_all_configs().get_global().0).expect("already should be parsed correctly")
        };
        // modify the settings for cfg file
        self.run(&mut cfg)
    }
}

impl Config {
    fn run(&self, _cfg: &mut config::Config) -> Result<(), Box<dyn std::error::Error>> {
        todo!("implement");
        // // check for list appending
        // for entry in &self.append {
        //     match entry.0.as_ref() {
        //         "include" => cfg.append_include(&entry.1),
        //         "vendor.index" => cfg.append_vendor_index(&entry.1),
        //         _ => return Err(AnyError(format!("unsupported key '{}' cannot be appended", entry.0)))?
        //     };
        // }
        // for entry in &self.set {
        //     // split by dots to get table.key (silently ignores improper parsing)
        //     if let Some((table, key)) = entry.0.split_once('.') {
        //         cfg.set(table, key, &entry.1)
        //     } else {
        //         return Err(AnyError(format!("unsupported key '{}' cannot be set", entry.0)))?
        //     }
        // }
        // for key in &self.unset {
        //     // split by dots to get table.key (silently ignores improper parsing)
        //     if let Some((table, key)) = key.split_once('.') {
        //         cfg.unset(table, key)?
        //     } else {
        //         return Err(AnyError(format!("unsupported key '{}' cannot be set", key)))?
        //     }
        // }
        // cfg.write()
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