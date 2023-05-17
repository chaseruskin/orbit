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
use crate::core::v2::manifest::Source;
use crate::core::v2::lockfile::LockFile;
use crate::core::v2::catalog::Catalog;
use crate::core::v2::lockfile::LockEntry;
use crate::util::anyerror::Fault;
use crate::core::v2::plugin::Process;
use crate::core::v2::protocol::Protocol;
use std::io::Write;
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

        let dl_proc = c.get_config().get_protocols().into_iter().next();

        // // do not allow args if no command is set
        // if dl_proc.is_none() == true {
        //     panic!("no protocol defined!")
        // }
        // if dl_proc.exists() == false && self.args.is_empty() == false {
        //     panic!("invalid arguments for no command set")
        // }
        
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
        let to_stdout = dl_proc.is_some() == false || self.list == true;

        let downloads =  Self::compile_download_list(&LockEntry::from(&ip), ip.get_lock(), &catalog, missing_only);
        // print to console
        if to_stdout == true {
            downloads.iter().for_each(|d| println!("{}", d));
        // execute the command
        } else {
            Self::download(&downloads, dl_proc.unwrap().1, &self.args, self.verbose)?;
        }
        Ok(())
    }
}

impl Download {
    /// Generates a list of dependencies required to be downloaded from the internet. 
    /// 
    /// Enabling `missing_only` will only push sources for ip not already installed.
    pub fn compile_download_list<'a>(le: &LockEntry, lf: &'a LockFile, catalog: &Catalog, missing_only: bool) -> Vec<&'a Source> {
        lf.inner().iter()
            .filter(|p| p.get_source().is_some() == true)
            .filter(|p| p.matches_target(&le) == false && (missing_only == false || catalog.is_cached_slot(&p.to_cache_slot_key()) == false))
            .map(|f| f.get_source().unwrap())
            .collect()
    }

    pub fn download(downloads: &Vec<&String>, dl_proc: &Protocol, extra_args: &Vec<String>, verbose: bool) -> Result<(), Fault> {
        // write the download list to a temporary file
        let mut file = tempfile::NamedTempFile::new()?;
        let contents = downloads.iter().fold(String::new(), |mut acc, x| { acc.push_str(&x); acc.push_str("\n"); acc });
        file.write(&contents.as_bytes())?;

        // set a new env var
        Environment::new()
            .add(EnvVar::new().key("ORBIT_DOWNLOAD_LIST").value(&file.path().to_string_lossy()))
            .initialize();
        
        let result = dl_proc.execute(&extra_args, verbose);
        // clean up temporary file
        file.close()?;
        result?;
        Ok(())
    }
}

const HELP: &str = "\
Request packages from the internet.

Usage:
    orbit download [options] [--] [args]...

Options:
    --command <cmd>     command to execute
    --list              print URLs to the console
    --missing           filter only uninstalled packages (default: true)
    --all               contain all packages in list
    --queue <dir>       set the destination queue directory
    --verbose           display the command being executed
    -- args...          arguments to pass to the requested command

Use 'orbit help download' to learn more about the command.
";