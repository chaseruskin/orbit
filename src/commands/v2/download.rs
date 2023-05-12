use std::path::PathBuf;

use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_QUEUE;
use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::OrbitResult;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::v2::manifest::Source;
use crate::core::plugin::Plugin;
use crate::core::v2::lockfile::LockFile;
use crate::core::v2::catalog::Catalog;
use std::error::Error;
use crate::core::v2::lockfile::LockEntry;

use crate::core::v2::ip::Ip;

#[derive(Debug, PartialEq)]
pub struct Download {
    all: bool,
    missing: bool,
    list: bool,
    command: Option<String>,
    queue_dir: Option<PathBuf>,
    args: Vec<String>,
    verbose: bool,
}

impl FromCli for Download {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Download {
            all: cli.check_flag(Flag::new("all"))?,
            missing: cli.check_flag(Flag::new("missing"))?,
            list: cli.check_flag(Flag::new("list"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            queue_dir: cli.check_option(Optional::new("queue").value("dir"))?,
            command: cli.check_option(Optional::new("command").value("cmd"))?,
            args: cli.check_remainder()?,
        });
        command
    }
}

impl Command<Context> for Download {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {

        // @idea: display lock entries as JSON? or use different env var for ORBIT_DOWNLOAD_LIST and ORBIT_VERSION_LIST

        // cannot happen
        if self.all == true && self.missing == true {
            panic!("cannot display all and missing lock entries");
        }

        // load the catalog
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .queue(&self.queue_dir.as_ref().unwrap_or(c.get_queue_path()))?;

        // verify running from an IP directory and enter IP's root directory
        c.goto_ip_path()?;

        let ip = Ip::load(c.get_ip_path().unwrap().clone())?;

        // verify a lockfile exists
        if ip.get_lock().is_empty() == true {
            panic!("cannot download due to missing lockfile")
        }

        // determine the queue directory based on cli priority
        let q_dir = self.queue_dir.as_ref().unwrap_or(c.get_queue_path());

        Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&Ip::load(c.get_ip_path().unwrap().clone())?)?
            .add(EnvVar::new().key(ORBIT_QUEUE).value(&q_dir.to_string_lossy()))
            .initialize();
        
        // default behavior is report only missing installations
        let missing_only = self.all == false || self.missing == true;

        // default behavior is to print out to console
        let to_stdout = self.command.is_none() || self.list == true;

        if to_stdout == true {
            for s in Self::compile_download_list(&ip, ip.get_lock(), &catalog, missing_only) {
                println!("{}", s);
            }
            return Ok(())
        }
        Ok(())
    }
}

impl Download {
    /// Generates a list of dependencies required to be downloaded from the internet. 
    /// 
    /// Enabling `missing_only` will only push sources for ip not already installed.
    pub fn compile_download_list<'a>(ip: &Ip, lf: &'a LockFile, catalog: &Catalog, missing_only: bool) -> Vec<&'a Source> {
        lf.inner().iter()
            .filter(|p| p.get_source().is_some() == true)
            .filter(|p| p.matches_target(&LockEntry::from(ip)) == false && (missing_only == false || catalog.is_cached_slot(&p.to_cache_slot_key()) == false))
            .map(|f| f.get_source().unwrap())
            .collect()
    }

    fn run(&self, plug: Option<&Plugin>) -> Result<(), Box<dyn Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args, self.verbose)
        } else if let Some(cmd) = &self.command {
            if self.verbose == true {
                let s = self.args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " });
                println!("running: {} {}", cmd, s);
            }
            let mut proc = crate::util::filesystem::invoke(cmd, &self.args, Context::enable_windows_bat_file_match())?;
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
    --build-dir <dir>   set the output build directory
    --verbose           display the command being executed
    -- args...          arguments to pass to the requested command

Use 'orbit help build' to learn more about the command.
";