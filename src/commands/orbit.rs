use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Positional};
use crate::interface::errors::CliError;
use crate::util::prompt;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct Orbit {
    help: bool,
    upgrade: bool,
    version: bool,
    force: bool,
    command: Option<OrbitSubcommand>,
}

impl Command for Orbit {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        self.run(context)
    }
}

use reqwest;
use std::env;

impl Orbit {
    fn run(&self, _: &Context) -> Result<(), Box<dyn std::error::Error>> {
        // prioritize version information
        if self.version {
            println!("orbit {}", VERSION);
            Ok(())
        // prioritize upgrade information
        } else if self.upgrade == true {
            println!("info: checking for latest orbit binary...");
            let info = self.upgrade()?;
            println!("info: {}", info);
            Ok(())
        // run the specified command
        } else if let Some(c) = &self.command {
            // set up the context (ignores the context passed in)
            let context = Context::new()
                .home("ORBIT_HOME")?
                .cache("ORBIT_CACHE")?
                .settings("config.toml")?
                .current_ip_dir("ORBIT_IP_PATH")?
                .build_dir("ORBIT_BUILD_DIR")?
                .development_path("ORBIT_PATH")?
                .retain_options(self.force);
            // pass the context to the given command
            c.exec(&context)
        // if no command is given then print default help
        } else {
            Ok(println!("{}", HELP))
        }
    }
}

impl FromCli for Orbit {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let orbit = Ok(Orbit {
            help: cli.check_flag(Flag::new("help").switch('h'))?,
            version: cli.check_flag(Flag::new("version"))?,
            upgrade: cli.check_flag(Flag::new("upgrade"))?,
            force: cli.check_flag(Flag::new("force"))?,
            command: cli.check_command(Positional::new("command"))?,
        });
        orbit
    }
}

use crate::commands::help::Help;
use crate::commands::new::New;
use crate::commands::search::Search;
use crate::commands::plan::Plan;
use crate::commands::build::Build;
use crate::commands::edit::Edit;
use crate::commands::launch::Launch;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
    New(New),
    Search(Search),
    Plan(Plan),
    Build(Build),
    Edit(Edit),
    Launch(Launch),
}

impl FromCli for OrbitSubcommand {
    fn from_cli<'c>(cli: &'c mut Cli<'_>) -> Result<Self, CliError<'c>> { 
        match cli.match_command(&[
            "help",
            "new",
            "search",
            "plan",
            "build",
            "edit",
            "launch",
        ])?.as_ref() {
            "help" => Ok(OrbitSubcommand::Help(Help::from_cli(cli)?)),
            "new" => Ok(OrbitSubcommand::New(New::from_cli(cli)?)),
            "search" => Ok(OrbitSubcommand::Search(Search::from_cli(cli)?)),
            "plan" => Ok(OrbitSubcommand::Plan(Plan::from_cli(cli)?)),
            "build" => Ok(OrbitSubcommand::Build(Build::from_cli(cli)?)),
            "edit" => Ok(OrbitSubcommand::Edit(Edit::from_cli(cli)?)),
            "launch" => Ok(OrbitSubcommand::Launch(Launch::from_cli(cli)?)),
            _ => panic!("an unimplemented command was passed through!")
        }
    }
}

impl Command for OrbitSubcommand {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        match self {
            OrbitSubcommand::Search(c) => c.exec(context),
            OrbitSubcommand::Plan(c) => c.exec(context),
            OrbitSubcommand::Build(c) => c.exec(context),
            OrbitSubcommand::Help(c) => c.exec(context),
            OrbitSubcommand::New(c) => c.exec(context),
            OrbitSubcommand::Edit(c) => c.exec(context),
            OrbitSubcommand::Launch(c) => c.exec(context),
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
    edit            open an ip in a text editor
    tree            view the dependency graph
    plan            generate a blueprint file
    build           execute a plugin
    launch          release a new ip version
    search          browse the ip catalog 

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
";

use crate::core::version;
use crate::util::sha256;
use std::str::FromStr;
use std::io::Write;
use zip;
use tempfile;

impl Orbit {
    #[tokio::main]
    async fn upgrade(&self) -> Result<String, Box<dyn std::error::Error>> {
        // check for stale versions at the current executable's path
        let exe_path = get_exe_path()?;
        let mut current_exe_dir = exe_path.clone();
        current_exe_dir.pop();
        // find any old versions existing in executable's current folder
        let paths = std::fs::read_dir(&current_exe_dir)?;
        for path in paths {
            if path.as_ref().unwrap().path().file_name().unwrap().to_str().unwrap().starts_with("orbit-") {
                // remove stale binaries
                std::fs::remove_file(path.as_ref().unwrap().path())?;
            }
        }

        // check the connection to grab latest html data
        let base_url: &str = "https://github.com/c-rus/orbit/releases";
        let res = reqwest::get(base_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedConnection(base_url.to_string(), res.status())))?
        }

        // create body into string to find the latest version
        let body = res.text().await?;
        let key = "href=\"/c-rus/orbit/releases/tag/";

        // if cannot find the key then the releases page failed to auto-detect in html data
        let pos = match body.find(&key) {
            Some(r) => r,
            None => return Err(Box::new(UpgradeError::NoReleasesFound))?
        };
        // assumes tag is complete version i.e. 1.0.0
        let (_, sub) = body.split_at(pos+key.len());
        let (version, _) = sub.split_once('\"').unwrap();

        // our current version is guaranteed to be valid
        let current = version::Version::from_str(VERSION).unwrap();
        // the latest version 
        let latest = version::Version::from_str(version).expect("invalid version released");
        if latest > current {
            // await user input
            if self.force == false {
                if prompt::prompt(&format!("info: a new version is available ({}), would you like to upgrade", latest))? == false {
                    return Ok(String::from("upgrade cancelled"))
                }
            }
        } else {
            return Ok(format!("the latest version is already installed ({})", &latest));
        }

        // download the list of checksums
        let sum_url = format!("{0}/download/{1}/orbit-{1}-checksums.txt", &base_url, &latest);
        let res = reqwest::get(&sum_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(sum_url.to_string(), res.status())))?
        }

        // store user's target
        let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        
        let checksums = String::from_utf8(res.bytes().await?.to_vec())?;
        let pkg = format!("orbit-{}-{}.zip", &latest, &target);
        // search the checksums to check if the desired pkg is available for download
        let cert = checksums.split_terminator('\n').find_map(|p| {
            let (cert, key) = p.split_once(' ').expect("bad checksum file format");
            if key == pkg  {
                Some(sha256::Sha256Hash::from_str(cert).expect("bad checksum format"))
            } else {
                None
            }
        });
        // verify there was a certificate/checksum for the requested pkg
        let cert = match cert {
            Some(c) => c,
            None => return Err(Box::new(UpgradeError::UnsupportedTarget(target)))?,
        };

        // download the zip pkg file
        let pkg_url = format!("{}/download/{}/{}",&base_url, &latest, &pkg);
        let res = reqwest::get(&pkg_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(pkg_url.to_string(), res.status())))?
        }
        let body_bytes = res.bytes().await?;

        // compute the checksum on the downloaded zip file
        let sum = sha256::compute_sha256(&body_bytes);
        // verify the checksums match
        match sum == cert {
            true => println!("info: verified download"),
            false =>  return Err(Box::new(UpgradeError::BadChecksum))?,
        };

        // unzip the bytes and put file in temporary file
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&body_bytes)?;
        let mut zip_archive = zip::ZipArchive::new(temp_file)?;

        // decompress zip file to a temporary directory
        let temp_dir = tempfile::tempdir()?;
        zip_archive.extract(&temp_dir)?;

        let exe_ext = if std::env::consts::EXE_EXTENSION.is_empty() == false { "" } else { ".exe" };

        // verify the path to the new executable exists before renaming current binary
        let temp_exe_path = temp_dir.path().join(&format!("orbit-{}-{}/bin/orbit{}", &latest, &target, &exe_ext));
        if std::path::Path::exists(&temp_exe_path) == false {
            return Err(Box::new(UpgradeError::MissingExe))?;
        }

        // rename the current binary with its version to become a 'stale binary'
        let stale_exe_path = current_exe_dir.join(&format!("orbit-{}", VERSION));
        std::fs::rename(&exe_path, &stale_exe_path)?;

        // copy the executable from the temporary directory to the original location
        std::fs::copy(&temp_exe_path, &exe_path)?;

        Ok(String::from(format!("successfully upgraded orbit to version {}", &latest)))
    }
}

#[derive(Debug, PartialEq)]
enum UpgradeError {
    UnsupportedTarget(String),
    FailedConnection(String, reqwest::StatusCode),
    FailedDownload(String, reqwest::StatusCode),
    NoReleasesFound,
    BadChecksum,
    MissingExe,
}

impl std::error::Error for UpgradeError {}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::MissingExe => write!(f, "failed to find the binary in the downloaded package"),
            Self::BadChecksum => write!(f, "checksums did not match, please try again"),
            Self::FailedConnection(url, status) => write!(f, "connection failed\n\nurl: {}\nstatus: {}", url, status),
            Self::FailedDownload(url, status) => write!(f, "download failed\n\nurl: {}\nstatus: {}", url, status),
            Self::UnsupportedTarget(t) => write!(f, "no pre-compiled binaries exist for the current target {}", t),
            Self::NoReleasesFound => write!(f, "no releases were found"),
        }
    }
}

use std::path::PathBuf;

fn get_exe_path() -> Result<PathBuf, Box::<dyn std::error::Error>> {
    match env::current_exe() {    
        Ok(exe_path) => Ok(std::fs::canonicalize(exe_path)?),
        Err(e) => Err(Box::new(e)),
    }
}