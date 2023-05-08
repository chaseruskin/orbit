use clif::cmd::Command;
use clif::cmd::FromCli;
use crate::core::lang::vhdl::highlight::ColorMode;
use clif::arg::Optional;
use clif::Cli;
use clif::arg::{Flag, Positional};
use clif::Error as CliError;
use crate::util::environment;
use crate::util::prompt;
use crate::core::context::Context;
use crate::util::sha256::Sha256Hash;
use std::env;

pub type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;

pub type OrbitResult = AnyResult<()>;

#[derive(Debug, PartialEq)]
pub struct Orbit {
    help: bool,
    upgrade: bool,
    version: bool,
    force: bool,
    command: Option<OrbitSubcommand>,
}

impl Command<()> for Orbit {
    type Status = OrbitResult;

    fn exec(&self, context: &()) -> Self::Status {
        self.run(context)
    }
}

impl Orbit {
    fn run(&self, _: &()) -> OrbitResult {
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
                .queue(environment::ORBIT_QUEUE)?
                .store(environment::ORBIT_STORE)?
                .current_ip_dir(environment::ORBIT_IP_PATH)? // must come before .settings() call
                .settings(crate::core::config::CONFIG_FILE)?
                .build_dir(environment::ORBIT_BUILD_DIR)?
                .development_path(environment::ORBIT_DEV_PATH, c.bypass_check() == false)?
                .read_vendors()?;
            // pass the context to the given command
            c.exec(&context)
        // if no command is given then print default help
        } else {
            Ok(println!("{}", HELP))
        }
    }
}

impl FromCli for Orbit {
    fn from_cli(cli: &mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        // need to set this coloring mode ASAP
        match cli.check_option(Optional::new("color").value("when"))?.unwrap_or(ColorMode::Auto) {
            ColorMode::Always => {
                cli.use_color();
                colored::control::set_override(true);
            },
            ColorMode::Never => {
                cli.disable_color();
                colored::control::set_override(false);
            },
            ColorMode::Auto => (),
        }
        let orbit = Ok(Orbit {
            help: cli.check_flag(Flag::new("help").switch('h'))?,
            upgrade: cli.check_flag(Flag::new("upgrade"))?,
            version: cli.check_flag(Flag::new("version"))?,
            force: cli.check_flag(Flag::new("force"))?,
            command: cli.check_command(Positional::new("command"))?,
        });
        // verify there are zero unhandled arguments
        cli.is_empty()?;
        orbit
    }
}

use crate::commands::help::Help;
use crate::commands::v2::new::New;
use crate::commands::v2::init::Init;
use crate::commands::v2::get::Get;
use crate::commands::v2::show::Show;
use crate::commands::v2::install::Install;
use crate::commands::v2::plan::Plan;
use crate::commands::search::Search;
use crate::commands::build::Build;
use crate::commands::launch::Launch;
use crate::commands::tree::Tree;
use crate::commands::env::Env;
use crate::commands::config::Config;
use crate::commands::uninstall::Uninstall;
use crate::commands::read::Read;

#[derive(Debug, PartialEq)]
enum OrbitSubcommand {
    Help(Help),
    New(New),
    Search(Search),
    Plan(Plan),
    Build(Build),
    Launch(Launch),
    Install(Install),
    Tree(Tree),
    Get(Get),
    Init(Init),
    Show(Show),
    Env(Env),
    Config(Config),
    Uninstall(Uninstall),
    Read(Read),
}

impl FromCli for OrbitSubcommand {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> { 
        match cli.match_command(&[
            "help",
            "new",
            "search",
            "plan",
            "p",
            "build",
            "launch",
            "install",
            "get",
            "init",
            "tree",
            "show",
            "b",
            "env",
            "config",
            "uninstall",
            "read",
        ])?.as_ref() {
            "get" => Ok(OrbitSubcommand::Get(Get::from_cli(cli)?)),
            "help" => Ok(OrbitSubcommand::Help(Help::from_cli(cli)?)),
            "new" => Ok(OrbitSubcommand::New(New::from_cli(cli)?)),
            "search" => Ok(OrbitSubcommand::Search(Search::from_cli(cli)?)),
      "p" | "plan" => Ok(OrbitSubcommand::Plan(Plan::from_cli(cli)?)),
      "b" | "build" => Ok(OrbitSubcommand::Build(Build::from_cli(cli)?)),
            "init" => Ok(OrbitSubcommand::Init(Init::from_cli(cli)?)),
            "launch" => Ok(OrbitSubcommand::Launch(Launch::from_cli(cli)?)),
            "install" => Ok(OrbitSubcommand::Install(Install::from_cli(cli)?)),
            "tree" => Ok(OrbitSubcommand::Tree(Tree::from_cli(cli)?)),
            "show" => Ok(OrbitSubcommand::Show(Show::from_cli(cli)?)),
            "env" => Ok(OrbitSubcommand::Env(Env::from_cli(cli)?)),
            "config" => Ok(OrbitSubcommand::Config(Config::from_cli(cli)?)),
            "uninstall" => Ok(OrbitSubcommand::Uninstall(Uninstall::from_cli(cli)?)),
            "read" => Ok(OrbitSubcommand::Read(Read::from_cli(cli)?)),
            _ => panic!("an unimplemented command was passed through!")
        }
    }
}

impl OrbitSubcommand {
    fn bypass_check(&self) -> bool {
        match self {
            Self::Config(_) => true,
            _ => false,
        }
    }
}

impl Command<Context> for OrbitSubcommand {
    type Status = OrbitResult;

    fn exec(&self, context: &Context) -> Self::Status {
        match self {
            OrbitSubcommand::Get(c) => c.exec(context),
            OrbitSubcommand::Search(c) => c.exec(context),
            OrbitSubcommand::Plan(c) => c.exec(context),
            OrbitSubcommand::Build(c) => c.exec(context),
            OrbitSubcommand::Install(c) => c.exec(context),
            OrbitSubcommand::Help(c) => c.exec(&()),
            OrbitSubcommand::New(c) => c.exec(&()),
            OrbitSubcommand::Launch(c) => c.exec(context),
            OrbitSubcommand::Tree(c) => c.exec(context),
            OrbitSubcommand::Init(c) => c.exec(context),
            OrbitSubcommand::Show(c) => c.exec(context),
            OrbitSubcommand::Env(c) => c.exec(context),
            OrbitSubcommand::Config(c) => c.exec(context),
            OrbitSubcommand::Uninstall(c) => c.exec(context),
            OrbitSubcommand::Read(c) => c.exec(context),
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
    show            print information about an ip
    read            inspect hdl design unit source code
    get             fetch an entity
    tree            view the dependency graph
    plan, p         generate a blueprint file
    build, b        execute a plugin
    search          browse the ip catalog 
    install         store an immutable reference to an ip
    env             print Orbit environment information
    config          modify configuration values
    uninstall       remove an ip from the catalog

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --color <when>  coloring: auto, always, never
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
";

/*
--*-- commands to remove --*--
    edit            open an ip in a text editor
    launch          release a new ip version

--*-- commands to add --*--
    audit           verify a package is able to installed properly and release-able
    run             perform plan and build in same step


alt names for `probe`: -check-, -scan-, show
*/ 

use crate::core::version;
use crate::util::sha256;
use std::str::FromStr;
use std::io::Write;
use zip;
use tempfile;
use crate::util::filesystem::get_exe_path;
use curl::easy::{Easy, List};

use serde_json::Value;

const RESPONSE_OKAY: u32 = 200;

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
    fn upgrade(&self) -> Result<String, Box<dyn std::error::Error>> {
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
        let api_url: &str = "https://api.github.com/repos/c-rus/orbit/releases/latest";

        let mut dst = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(api_url).unwrap();
            easy.follow_location(false).unwrap();
            // create headers
            let mut list = List::new();
            list.append("User-Agent: Orbit").unwrap();
            easy.http_headers(list).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer.write_function(|data| {
                    dst.extend_from_slice(data);
                    Ok(data.len())
                }).unwrap();
        
                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(api_url.to_owned(), rc)));
            } 
        }
        let body: String = String::from_utf8(dst)?;
        
        // create body into string to find the latest version
        let version = {
            let json_word: Value = serde_json::from_str(body.as_ref())?;
            json_word["name"].as_str().unwrap().to_string()
        };

        // our current version is guaranteed to be valid
        let current = version::Version::from_str(VERSION).unwrap();
        // the latest version 
        let latest = version::Version::from_str(&version).expect("invalid version released");
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

        let base_url: &str = "https://github.com/c-rus/orbit/releases";

        // download the list of checksums
        println!("info: downloading update...");
        let sum_url = format!("{0}/download/{1}/orbit-{1}-checksums.txt", &base_url, &latest);

        let mut dst = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&sum_url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer.write_function(|data| {
                    dst.extend_from_slice(data);
                    Ok(data.len())
                }).unwrap();
        
                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(sum_url, rc)));
            } 
        }
        let checksums: String = String::from_utf8(dst)?;
        
        // store user's target
        let target = Orbit::target_triple();
        
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
        // let res = reqwest::get(&pkg_url).await?;
        // if res.status() != 200 {
        //     return Err(Box::new(UpgradeError::FailedDownload(pkg_url.to_string(), res.status())))?
        // }
        // let body_bytes = res.bytes().await?;

        let mut body_bytes = Vec::new();
        {
            let mut easy = Easy::new();
            easy.url(&pkg_url).unwrap();
            easy.follow_location(true).unwrap();
            {
                let mut transfer = easy.transfer();
                transfer.write_function(|data| {
                    body_bytes.extend_from_slice(data);
                    Ok(data.len())
                }).unwrap();
        
                transfer.perform()?;
            }
            let rc = easy.response_code()?;
            if rc != RESPONSE_OKAY {
                return Err(Box::new(UpgradeError::FailedConnection(pkg_url, rc)));
            } 
        }

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
    FailedConnection(String, u32),
    FailedDownload(String, u32),
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