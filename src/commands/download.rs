use std::path::PathBuf;

use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::lockfile::LockEntry;
use crate::core::lockfile::LockFile;
use crate::core::plugin::Process;
use crate::core::protocol::Protocol;
use crate::core::source::Source;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_QUEUE;
use crate::OrbitResult;
use clif::arg::{Flag, Optional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Download {
    all: bool,
    missing: bool,
    list: bool,
    queue_dir: Option<PathBuf>,
    verbose: bool,
    force: bool,
}

impl FromCli for Download {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Download {
            all: cli.check_flag(Flag::new("all"))?,
            missing: cli.check_flag(Flag::new("missing"))?,
            list: cli.check_flag(Flag::new("list"))?,
            force: cli.check_flag(Flag::new("force"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            queue_dir: cli.check_option(Optional::new("queue").value("dir"))?,
        });
        command
    }
}

pub type ProtocolMap<'a> = HashMap<&'a str, &'a Protocol>;

impl Command<Context> for Download {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // @idea: display lock entries as JSON? or use different env var for ORBIT_DOWNLOAD_LIST and ORBIT_VERSION_LIST

        // cannot happen
        if self.all == true && self.missing == true {
            panic!("cannot display all and missing lock entries");
        }

        let proto_map: ProtocolMap = c.get_config().get_protocols();

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
            .add(
                EnvVar::new()
                    .key(ORBIT_QUEUE)
                    .value(&q_dir.to_string_lossy()),
            )
            .initialize();

        // default behavior is report only missing installations
        let missing_only = self.all == false || self.missing == true;

        // default behavior is to print out to console
        let to_stdout = self.list == true;

        let downloads = Self::compile_download_list(
            &LockEntry::from((&ip, true)),
            ip.get_lock(),
            &catalog,
            missing_only,
        );
        // print to console
        if to_stdout == true {
            downloads.iter().for_each(|(_, src)| println!("{}", src));
        // execute the command
        } else {
            Self::download_all(
                &downloads,
                &proto_map,
                self.verbose,
                c.get_queue_path(),
                self.force,
            )?;
        }
        Ok(())
    }
}

impl Download {
    /// Generates a list of dependencies required to be downloaded from the internet.
    ///
    /// Enabling `missing_only` will only push sources for ip not already installed.
    pub fn compile_download_list<'a>(
        le: &LockEntry,
        lf: &'a LockFile,
        catalog: &Catalog,
        missing_only: bool,
    ) -> Vec<(IpSpec, &'a Source)> {
        lf.inner()
            .iter()
            .filter(|p| p.get_source().is_some() == true)
            .filter(|p| {
                p.matches_target(&le) == false
                    && (missing_only == false
                        || catalog.is_cached_slot(&p.to_cache_slot_key()) == false)
            })
            .map(|f| (f.to_ip_spec(), f.get_source().unwrap()))
            .collect()
    }

    pub fn download(
        spec: &IpSpec,
        src: &Source,
        queue: &PathBuf,
        protocols: &HashMap<&str, &Protocol>,
        verbose: bool,
        force: bool,
    ) -> Result<(), Fault> {
        // access the protocol
        if let Some(proto) = src.get_protocol() {
            match protocols.get(proto.as_str()) {
                Some(entry) => {
                    println!(
                        "info: Downloading {} over \"{}\" protocol ...",
                        spec, &proto
                    );
                    entry.execute(&[src.get_url().to_string()], verbose)?
                }
                None => {
                    if force == false {
                        return Err(
                            Box::new(AnyError(format!("Unknown protocol \"{}\"", &proto))).into(),
                        );
                    }
                }
            }
        }
        // try to use default protocol
        if force == true || src.is_default() == true {
            println!("info: Downloading {} ...", spec);
            Protocol::single_download(src.get_url(), queue)?;
        }
        Ok(())
    }

    pub fn download_all(
        downloads: &Vec<(IpSpec, &Source)>,
        proto_map: &HashMap<&str, &Protocol>,
        verbose: bool,
        queue: &PathBuf,
        force: bool,
    ) -> Result<(), Fault> {
        match downloads.len() {
            0 => {
                println!("info: No missing downloads");
                return Ok(());
            }
            1 => {
                println!("info: Downloading 1 package ...")
            }
            _ => {
                println!("info: Downloading {} packages ...", downloads.len())
            }
        }

        let mut results = downloads.iter().filter_map(|e| {
            match Self::download(&e.0, &e.1, &queue, &proto_map, verbose, force) {
                Ok(_) => None,
                Err(e) => Some(e),
            }
        });
        if let Some(n) = results.next() {
            return Err(n);
        }

        Ok(())
    }
}

const HELP: &str = "\
Request packages from the internet.

Usage:
    orbit download [options]

Options:
    --list              print URLs to the console
    --missing           filter only uninstalled packages (default: true)
    --all               contain all packages in list
    --queue <dir>       set the destination queue directory
    --verbose           display the command being executed
    --force             fallback to default protocol if missing given protocol

Use 'orbit help download' to learn more about the command.
";
