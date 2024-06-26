use std::path::PathBuf;
use std::str::FromStr;

use crate::commands::helps::config;
use crate::core;
use crate::core::config::ConfigDocument;
use crate::core::config::CONFIG_FILE;
use crate::core::context::Context;
use crate::core::manifest::FromFile;
use crate::error::Error;
use crate::error::LastError;
use crate::util::anyerror::AnyError;
use colored::*;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

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
    pop: Vec<String>,
    set: Vec<Entry>,
    unset: Vec<String>,
}

impl Subcommand<Context> for Config {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(config::HELP))?;
        Ok(Config {
            // Flags
            global: cli.check(Arg::flag("global"))?,
            local: cli.check(Arg::flag("local"))?,
            // Options
            append: cli
                .get_all(Arg::option("append").value("key=value"))?
                .unwrap_or(Vec::new()),
            pop: cli
                .get_all(Arg::option("pop").value("key"))?
                .unwrap_or(Vec::new()),
            set: cli
                .get_all(Arg::option("set").value("key=value"))?
                .unwrap_or(Vec::new()),
            unset: cli
                .get_all(Arg::option("unset").value("key"))?
                .unwrap_or(Vec::new()),
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
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
    fn no_options_selected(&self) -> bool {
        self.append.is_empty()
            && self.pop.is_empty()
            && self.set.is_empty()
            && self.unset.is_empty()
    }

    fn run(
        &self,
        config: &mut ConfigDocument,
        file: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // display configuration and exit
        if self.no_options_selected() == true {
            println!("{}", config.to_string());
            return Ok(());
        }

        // check for list appending
        for entry in &self.append {
            match entry.0.as_ref() {
                "include" => config.append_include(&entry.1),
                "general.languages" => config.append_languages(&entry.1),
                _ => return Err(Error::ConfigFieldNotList(entry.0.to_string()))?,
            };
        }
        // check list for popping
        for key in &self.pop {
            match key.as_ref() {
                "include" => config.pop_include(),
                "general.languages" => config.pop_languages(),
                _ => return Err(Error::ConfigFieldNotList(key.to_string()))?,
            };
        }

        for entry in &self.set {
            // split by dots to get table.key (silently ignores improper parsing)
            if let Some((table, key)) = entry.0.split_once('.') {
                config.set(table, key, &entry.1)
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
                config.unset(table, key)?
            } else {
                return Err(AnyError(format!("unsupported key '{}' cannot be set", key)))?;
            }
        }

        // is the config file is okay?
        if let Err(e) = core::config::Config::from_str(&config.to_string()) {
            return Err(Error::ConfigNotSaved(LastError(e.to_string())))?;
        }

        config.write(&file)
    }
}
