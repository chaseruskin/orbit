use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Positional};
use crate::interface::errors::CliError;


#[derive(Debug, PartialEq)]
pub struct Orbit {
    help: bool,
    upgrade: bool,
    version: bool,
    command: Option<OrbitSubcommand>,
}

impl Command for Orbit {
    fn exec(&self) -> () {
        self.run();
    }
}

use reqwest;
use std::io::prelude::*;

impl Orbit {
    fn run(&self) -> () {
        // prioritize version information
        if self.version {
            println!("orbit {}", VERSION);
        // prioritize upgrade information
        } else if self.upgrade == true {
            match Self::upgrade() {
                Ok(_) => (),
                Err(e) => eprintln!("upgrade-error: {}", e),
            }
        // run the specified command
        } else if let Some(c) = &self.command {
            c.exec();
        // if no command is given then print default help
        } else {
            println!("{}", HELP);
        }
    }

    fn upgrade() -> Result<(), String> {
        println!("info: checking for latest orbit binary...");
        match Self::connect() {
            Ok(()) => (),
            Err(e) => eprintln!("error: {}", e),
        }
        Ok(())
    }
}

impl FromCli for Orbit {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let orbit = Ok(Orbit {
            help: cli.check_flag(Flag::new("help").switch('h'))?,
            version: cli.check_flag(Flag::new("version"))?,
            upgrade: cli.check_flag(Flag::new("upgrade"))?,
            command: cli.check_command(Positional::new("command"))?,
        });
        orbit
    }
}

use crate::commands::help::Help;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
}

impl FromCli for OrbitSubcommand {
    fn from_cli<'c>(cli: &'c mut Cli<'_>) -> Result<Self, CliError<'c>> { 
        match cli.match_command(&["help"])?.as_ref() {
            "help" => Ok(OrbitSubcommand::Help(Help::from_cli(cli)?)),
            _ => panic!("an unimplemented command was passed through!")
        }
    }
}

impl Command for OrbitSubcommand {
    fn exec(&self) {
        match self {
            OrbitSubcommand::Help(c) => c.exec(),
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
// :todo: check for additional data such as the commit being used

const HELP: &str = "\
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
";


use crate::core::version;
use std::str::FromStr;

impl Orbit {
    #[tokio::main]
    async fn connect() -> Result<(), Box<dyn std::error::Error>> {
        // bail early if the user has a unsupported os
        let os: &str = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            return Err(Box::new(UpgradeError::UnsupportedOS))?
        };

        // check the connection to grab latest html data
        let url: &str = "https://github.com/c-rus/guessing-game/releases";
        let res = reqwest::get(url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedConnection(url.to_string(), res.status())))?
        }

        // create body into string to find the latest version
        let body = res.text().await?;
        let key = "href=\"/c-rus/guessing-game/releases/tag/";

        // if cannot find the key then the releases page failed to auto-detect in html data
        let pos = match body.find(&key) {
            Some(r) => r,
            None => return Err(Box::new(UpgradeError::NoReleasesFound))?
        };
        // +1 to drop the leading 'v' with a version tag
        let (_, sub) = body.split_at(pos+key.len()+1);
        let (version, _) = sub.split_once('\"').unwrap();

        // our current version is guaranteed to be valid
        let current = version::Version::from_str(VERSION).unwrap();
        // the latest version 
        let latest = version::Version::from_str(version).expect("invalid version released");
        if latest > current {
            println!("info: a new version is available ({}), would you like to upgrade? [y/n]", latest);
        } else {
            println!("info: you have the latest version already ({}).", latest);
            return Ok(());
        }

        let url = format!("{}/download/v{}/guessing-game-{}-x64.zip",&url, &version, &os);
        let res = reqwest::get(&url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(url.to_string(), res.status())))?
        }

        let body_bytes = res.bytes().await?;
        // write the bytes to a file
        let mut file = std::fs::File::create("./upgrade.zip")?;
        file.write_all(&body_bytes)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum UpgradeError {
    UnsupportedOS,
    FailedConnection(String, reqwest::StatusCode),
    FailedDownload(String, reqwest::StatusCode),
    NoReleasesFound,
}

impl std::error::Error for UpgradeError {}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::FailedConnection(url, status) => write!(f, "connection to internet failed for request\n\nurl: {}\nstatus: {}", url, status),
            Self::FailedDownload(url, status) => write!(f, "download failed for request\n\nurl: {}\nstatus: {}", url, status),
            Self::UnsupportedOS => write!(f, "no pre-compiled binaries exist for your operating system"),
            Self::NoReleasesFound => write!(f, "no releases were found"),
        } 
    }
}