use std::path::PathBuf;
use std::str::FromStr;

use crate::commands::helps::config;
use crate::core::config::ConfigDocument;
use crate::core::config::CONFIG_FILE;
use crate::core::context::Context;
use crate::core::manifest::FromFile;
use crate::util::anyerror::AnyError;
use crate::OrbitResult;
use clif::arg::{Flag, Optional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use colored::*;

#[derive(Debug, PartialEq)]
pub struct Entry(String, String);

impl FromStr for Entry {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split on first '=' sign
        match s.split_once('=') {
            Some(e) => Ok(Entry(e.0.to_owned(), e.1.to_owned())),
            None => Err(AnyError(format!("missing '=' separator"))),
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
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(config::HELP).ref_usage(2..4))?;
        let command = Ok(Config {
            // Flags
            global: cli.check_flag(Flag::new("global"))?,
            local: cli.check_flag(Flag::new("local"))?,
            // Options
            append: cli
                .check_option_all(Optional::new("append").value("key=value"))?
                .unwrap_or(Vec::new()),
            set: cli
                .check_option_all(Optional::new("set").value("key=value"))?
                .unwrap_or(Vec::new()),
            unset: cli
                .check_option_all(Optional::new("unset").value("key"))?
                .unwrap_or(Vec::new()),
        });
        command
    }
}

impl Command<Context> for Config {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // check if we are using global or local
        if self.local == true && self.global == true {
            return Err(AnyError(format!(
                "'{}' and '{}' cannot be set at the same time",
                "--local".yellow(),
                "--global".yellow()
            )))?;
        }
        let (mut cfg, file) = if self.local == true {
            match c.get_ip_path() {
                Some(path) => {
                    let file = path.join(".orbit").join(CONFIG_FILE);
                    (ConfigDocument::from_file(&file)?, file)
                }
                None => {
                    return Err(AnyError(format!(
                        "no ip detected in the current directory to modify local configurations"
                    )))?
                }
            }
        } else {
            // duplicate the configuration so we can potentially mutate it
            let file = c.get_all_configs().get_global().0.clone();
            (
                ConfigDocument::from_file(&file).expect("already should be parsed correctly"),
                file,
            )
        };
        // modify the settings for cfg file
        self.run(&mut cfg, &file)
    }
}

impl Config {
    fn run(
        &self,
        cfg: &mut ConfigDocument,
        file: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // check for list appending
        for entry in &self.append {
            match entry.0.as_ref() {
                "include" => cfg.append_include(&entry.1),
                _ => {
                    return Err(AnyError(format!(
                        "unsupported key '{}' cannot be appended",
                        entry.0
                    )))?
                }
            };
        }

        for entry in &self.set {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = entry.0.split_once('.') {
                cfg.set(table, key, &entry.1)
            } else {
                return Err(AnyError(format!(
                    "unsupported key '{}' cannot be set",
                    entry.0
                )))?;
            }
        }

        for key in &self.unset {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = key.split_once('.') {
                cfg.unset(table, key)?
            } else {
                return Err(AnyError(format!("unsupported key '{}' cannot be set", key)))?;
            }
        }

        cfg.write(&file)
    }
}
