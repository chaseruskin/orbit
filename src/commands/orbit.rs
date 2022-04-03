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
use colored::*;

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
            Err(e) => eprintln!("{} {}", "error:".red().bold(), e),
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
        //  https://github.com/c-rus/guessing-game/releases/download/v1.0.6/checksum.txt
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
    
        let pkg = format!("guessing-game-{}-x64.zip", &os);
        // download the zip file
        let pkg_url = format!("{}/download/v{}/{}",&url, &version, &pkg);
        let res = reqwest::get(&pkg_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(pkg_url.to_string(), res.status())))?
        }
        let body_bytes = res.bytes().await?;
        // compute the checksum on the downloaded zip file
        let sum = sha256::compute_sha256(&body_bytes);

        // download the list of checksums
        let sum_url = format!("{}/download/v{}/checksum.txt", &url, &version);
        let res = reqwest::get(&sum_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(sum_url.to_string(), res.status())))?
        }
        // search the checksums for the downloaded pkg and verify they match
        let checksums = String::from_utf8(res.bytes().await?.to_vec())?;
        checksums.split_terminator('\n').find_map(|p| {
            let (cert, key) = p.split_once(' ').expect("bad checksum file format");
            if key == pkg  {
                match sum == sha256::Sha256Hash::from_str(cert).expect("bad checksum") {
                    true => Some(Ok(())),
                    false => Some(Err(UpgradeError::BadChecksum))
                }
            } else {
                None
            }
        }).expect("missing package in checksum.txt")?;
        println!("info: verified download");

        // unzip the bytes and then write to a file and rename current exe
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&body_bytes)?;
        let mut zip_archive = zip::ZipArchive::new(temp_file)?;
        // rename the current exe as format!("orbit-{}", VERSION)
        let current_exe_dir = "."; //todo find exe path/directory
        //zip_archive.extract(current_exe_dir)?;
        Ok(())
    }
}

use std::io::Write;
use zip;
use tempfile;

use crate::util::sha256;

#[derive(Debug, PartialEq)]
enum UpgradeError {
    UnsupportedOS,
    FailedConnection(String, reqwest::StatusCode),
    FailedDownload(String, reqwest::StatusCode),
    NoReleasesFound,
    BadChecksum,
}

impl std::error::Error for UpgradeError {}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::BadChecksum => write!(f, "checksums did not match, please try again"),
            Self::FailedConnection(url, status) => write!(f, "connection failed\n\nurl: {}\nstatus: {}", url, status),
            Self::FailedDownload(url, status) => write!(f, "download failed\n\nurl: {}\nstatus: {}", url, status),
            Self::UnsupportedOS => write!(f, "no pre-compiled binaries exist for your operating system"),
            Self::NoReleasesFound => write!(f, "no releases were found"),
        } 
    }
}