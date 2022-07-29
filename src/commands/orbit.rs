use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Positional};
use crate::interface::errors::CliError;
use crate::util::environment;
use crate::util::prompt;
use crate::core::context::Context;
use crate::util::sha256::Sha256Hash;
use std::env;

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

impl Orbit {
    fn run(&self, _: &Context) -> Result<(), Box<dyn std::error::Error>> {
        // prioritize version information
        if self.version == true {
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
                .home(environment::ORBIT_HOME)?
                .cache(environment::ORBIT_CACHE)?
                .store(environment::ORBIT_STORE)?
                .current_ip_dir(environment::ORBIT_IP_PATH)? // must come before .settings() call
                .settings(crate::core::config::CONFIG_FILE)?
                .build_dir(environment::ORBIT_BUILD_DIR)?
                .development_path(environment::ORBIT_DEV_PATH)?
                .read_vendors()?
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
            upgrade: cli.check_flag(Flag::new("upgrade"))?,
            version: cli.check_flag(Flag::new("version"))?,
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
use crate::commands::install::Install;
use crate::commands::tree::Tree;
use crate::commands::get::Get;
use crate::commands::init::Init;
use crate::commands::probe::Probe;
use crate::commands::env::Env;
use crate::commands::config::Config;
use crate::commands::uninstall::Uninstall;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
    New(New),
    Search(Search),
    Plan(Plan),
    Build(Build),
    Edit(Edit),
    Launch(Launch),
    Install(Install),
    Tree(Tree),
    Get(Get),
    Init(Init),
    Probe(Probe),
    Env(Env),
    Config(Config),
    Uninstall(Uninstall),
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
            "install",
            "get",
            "init",
            "tree",
            "probe",
            "b",
            "env",
            "config",
            "uninstall",
        ])?.as_ref() {
            "get" => Ok(OrbitSubcommand::Get(Get::from_cli(cli)?)),
            "help" => Ok(OrbitSubcommand::Help(Help::from_cli(cli)?)),
            "new" => Ok(OrbitSubcommand::New(New::from_cli(cli)?)),
            "search" => Ok(OrbitSubcommand::Search(Search::from_cli(cli)?)),
            "plan" => Ok(OrbitSubcommand::Plan(Plan::from_cli(cli)?)),
      "b" | "build" => Ok(OrbitSubcommand::Build(Build::from_cli(cli)?)),
            "edit" => Ok(OrbitSubcommand::Edit(Edit::from_cli(cli)?)),
            "init" => Ok(OrbitSubcommand::Init(Init::from_cli(cli)?)),
            "launch" => Ok(OrbitSubcommand::Launch(Launch::from_cli(cli)?)),
            "install" => Ok(OrbitSubcommand::Install(Install::from_cli(cli)?)),
            "tree" => Ok(OrbitSubcommand::Tree(Tree::from_cli(cli)?)),
            "probe" => Ok(OrbitSubcommand::Probe(Probe::from_cli(cli)?)),
            "env" => Ok(OrbitSubcommand::Env(Env::from_cli(cli)?)),
            "config" => Ok(OrbitSubcommand::Config(Config::from_cli(cli)?)),
            "uninstall" => Ok(OrbitSubcommand::Uninstall(Uninstall::from_cli(cli)?)),
            _ => panic!("an unimplemented command was passed through!")
        }
    }
}

impl Command for OrbitSubcommand {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        match self {
            OrbitSubcommand::Get(c) => c.exec(context),
            OrbitSubcommand::Search(c) => c.exec(context),
            OrbitSubcommand::Plan(c) => c.exec(context),
            OrbitSubcommand::Build(c) => c.exec(context),
            OrbitSubcommand::Install(c) => c.exec(context),
            OrbitSubcommand::Help(c) => c.exec(context),
            OrbitSubcommand::New(c) => c.exec(context),
            OrbitSubcommand::Edit(c) => c.exec(context),
            OrbitSubcommand::Launch(c) => c.exec(context),
            OrbitSubcommand::Tree(c) => c.exec(context),
            OrbitSubcommand::Init(c) => c.exec(context),
            OrbitSubcommand::Probe(c) => c.exec(context),
            OrbitSubcommand::Env(c) => c.exec(context),
            OrbitSubcommand::Config(c) => c.exec(context),
            OrbitSubcommand::Uninstall(c) => c.exec(context),
        }
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
// @TODO check for additional data such as the commit being used

const HELP: &str = "\
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip
    init            initialize an ip from an existing project
    edit            open an ip in a text editor
    probe           access information about an ip
    get             fetch an entity
    tree            view the dependency graph
    plan            generate a blueprint file
    build, b        execute a plugin
    launch          release a new ip version
    search          browse the ip catalog 
    install         store an immutable reference to an ip
    env             print Orbit environment information
    config          modify configuration values
    uninstall       remove an ip from the catalog

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
";

use reqwest;
use crate::core::version;
use crate::util::sha256;
use std::str::FromStr;
use std::io::Write;
use zip;
use tempfile;
use crate::util::filesystem::get_exe_path;

impl Orbit {
    /// Returns current machine's target as `<arch>-<os>`.
    fn target_triple() -> String {
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    }

    /// Runs a process to check for an updated version of Orbit on GitHub to install.
    /// 
    /// Steps it follows:  
    /// 1. Removes any old version existing in executables' current folder
    /// 2. Gets website data from GitHub releases page to check for latest version
    /// 3. If new version, download checksum file and search for a compatible platform
    /// 4. Download compatible platform zip file and verify checksum matches
    /// 5. Unzip the file and replace the Orbit executable in-place.
    /// 6. Rename the old executable as `orbit-<version>`.
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
        println!("info: downloading update...");
        let sum_url = format!("{0}/download/{1}/orbit-{1}-checksums.txt", &base_url, &latest);
        let res = reqwest::get(&sum_url).await?;
        if res.status() != 200 {
            return Err(Box::new(UpgradeError::FailedDownload(sum_url.to_string(), res.status())))?
        }

        // store user's target
        let target = Orbit::target_triple();
        
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
            false =>  return Err(Box::new(UpgradeError::BadChecksum(sum, cert)))?,
        };

        // unzip the bytes and put file in temporary file
        println!("info: installing update...");
        let mut temp_file = tempfile::tempfile()?;
        temp_file.write_all(&body_bytes)?;
        let mut zip_archive = zip::ZipArchive::new(temp_file)?;

        // decompress zip file to a temporary directory
        let temp_dir = tempfile::tempdir()?;
        zip_archive.extract(&temp_dir)?;

        let exe_ext = if std::env::consts::EXE_EXTENSION.is_empty() == true { "" } else { ".exe" };

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
    BadChecksum(Sha256Hash, Sha256Hash),
    MissingExe,
}

impl std::error::Error for UpgradeError {}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::MissingExe => write!(f, "failed to find the binary in the downloaded package"),
            Self::BadChecksum(computed, ideal) => write!(f, "checksums did not match, please try again\n\ncomputed: {}\nexpected: {}", computed, ideal),
            Self::FailedConnection(url, status) => write!(f, "connection failed\n\nurl: {}\nstatus: {}", url, status),
            Self::FailedDownload(url, status) => write!(f, "download failed\n\nurl: {}\nstatus: {}", url, status),
            Self::UnsupportedTarget(t) => write!(f, "no pre-compiled binaries exist for the current target {}", t),
            Self::NoReleasesFound => write!(f, "no releases were found"),
        }
    }
}