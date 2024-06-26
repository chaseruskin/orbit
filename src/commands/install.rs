//! The installation process:
//! 1. Optionally ask for an ip to install (default: current working ip)
//! --!-- Get the folder and change directories to the folder requiring installation --!--
//! * at the point in the process when the program is at the folder, it is assumed all sub-deps are also already installed
//! 2. Write results from computing the available units for the package
//! 3. Write results for information about accessing C,I,S,A
//! 3. Verify a .lock file is available (is this needed? - do dependents even read from this?)
//! 4. Move only relevant files to artifact directory (no .git/, etc.)
//! 5. Compute checksum on entire directory
//! 6. Zip contents and store in "store" for future re-installation
//! 7. Place artifact directory in "cache" for catalog lookup
//!
//! One issue that remains is how to retrieve packages from online automatically.
//!
//! The download process:
//!     - write a lockfile
//!     - ...
//!

use super::plan::Plan;
use crate::commands::download::Download;
use crate::commands::download::ProtocolMap;
use crate::commands::helps::install;
use crate::commands::plan;
use crate::core::algo;
use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::PartialIpSpec;
use crate::core::iparchive::IpArchive;
use crate::core::lockfile::LockEntry;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::manifest::ORBIT_SUM_FILE;
use crate::core::protocol::Protocol;
use crate::core::protocol::ProtocolError;
use crate::core::source::Source;
use crate::core::variable::VariableTable;
use crate::core::version;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Install {
    ip: Option<PartialIpSpec>,
    url: Option<String>,
    path: Option<PathBuf>,
    protocol: Option<String>,
    tag: Option<String>,
    list: bool,
    force: bool,
    verbose: bool,
    all: bool,
}

impl Subcommand<Context> for Install {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(install::HELP))?;
        Ok(Install {
            // Flags
            force: cli.check(Arg::flag("force"))?,
            verbose: cli.check(Arg::flag("verbose"))?,
            all: cli.check(Arg::flag("all"))?,
            list: cli.check(Arg::flag("list"))?,
            // Options
            path: cli.get(Arg::option("path"))?,
            url: cli.get(Arg::option("url"))?,
            tag: cli.get(Arg::option("tag"))?,
            protocol: cli.get(Arg::option("protocol").value("name"))?,
            // Positionals
            ip: cli.get(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // locate the plugin
        let protocol = match &self.protocol {
            // verify the plugin alias matches
            Some(name) => match c.get_config().get_protocols().get(name.as_str()) {
                Some(&p) => Some(p),
                None => return Err(ProtocolError::Missing(name.to_string()))?,
            },
            None => None,
        };

        // display protocol list and exit
        if self.list == true {
            match protocol {
                // display entire contents about the particular plugin
                Some(proto) => println!("{}", proto),
                // display quick overview of all plugins
                None => println!(
                    "{}",
                    Protocol::list_protocols(
                        &mut c
                            .get_config()
                            .get_protocols()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Protocol>>()
                    )
                ),
            }
            return Ok(());
        }
        // gather the catalog (all manifests)
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // check if trying to download from the internet
        let target = if self.url.is_some() {
            Self::download_target_from_url(&self, c, &self.url.as_ref().unwrap())?;
            None
        // check if trying to download from local filesystem
        } else if self.path.is_some() || self.ip.is_none() {
            // verify the path points to a valid ip
            let search_path = filesystem::resolve_rel_path(
                &env::current_dir()?,
                &filesystem::into_std_str(
                    self.path.as_ref().unwrap_or(&PathBuf::from(".")).clone(),
                ),
            );

            // check if specifying an Ip
            let search_dir = PathBuf::standardize(PathBuf::from(search_path));
            let search_path = search_dir.join(IP_MANIFEST_FILE);

            // println!("{:?}", search_path);
            // look for IP along this path
            //let result = manifest::find_file(&search_path, IP_MANIFEST_FILE, true)?;
            // find the IP to match
            // println!("{:?}", result);
            let target = match &self.ip {
                Some(entry) => match search_path.exists() {
                    true => {
                        let ip = Ip::load(search_dir.to_path_buf(), true)?;
                        if ip.get_man().get_ip().get_name() == entry.get_name()
                            && (entry.get_version().is_latest()
                                || version::is_compatible(
                                    entry.get_version().as_specific().unwrap(),
                                    ip.get_man().get_ip().get_version(),
                                ))
                        {
                            Some(ip)
                        } else {
                            Err(AnyError(format!(
                                "Could not find ip \"{}\" at path \"{}\"",
                                entry,
                                filesystem::into_std_str(search_dir)
                            )))?
                        }
                    }
                    false => Err(AnyError(format!(
                        "Path \"{}\" does not contain an Orbit.toml file",
                        filesystem::into_std_str(search_dir)
                    )))?,
                },
                // make sure there is only 1 IP to download
                None => match search_path.exists() {
                    true => Some(Ip::load(search_dir.to_path_buf(), true)?),
                    false => Err(AnyError(format!(
                        "Path \"{}\" does not contain an Orbit.toml file",
                        filesystem::into_std_str(search_dir)
                    )))?,
                },
            };
            target
        // attempt to find the catalog
        } else {
            None
        };

        // update the downloads
        catalog = catalog.downloads(c.get_downloads_path())?;

        // use the catalog (if no path is provided)
        let target = if self.path.is_none() {
            if let Some(spec) = &self.ip {
                if let Some(lvl) = catalog.inner().get(spec.get_name()) {
                    if let Some(slot) = lvl.get(true, spec.get_version()) {
                        if let Some(bytes) = slot.get_mapping().as_bytes() {
                            // println!("{} {}", "using archive", slot.get_man().get_ip().into_ip_spec());
                            // place the dependency into a temporary directory
                            // @MARK: fix this to cleanup manually since we forced it into_path.
                            let dir = tempfile::tempdir()?.into_path();
                            if let Err(e) = IpArchive::extract(&bytes, &dir) {
                                fs::remove_dir_all(dir)?;
                                return Err(e);
                            }
                            // load the IP
                            let unzipped_ip = match Ip::load(dir.clone(), false) {
                                Ok(x) => x,
                                Err(e) => {
                                    fs::remove_dir_all(dir)?;
                                    return Err(e);
                                }
                            };
                            Some(unzipped_ip)
                        } else {
                            Some(Ip::load(slot.get_root().clone(), false)?)
                        }
                    } else {
                        return Err(AnyError(format!(
                            "ip {} does not exist in the catalog",
                            spec
                        )))?;
                    }
                } else {
                    return Err(AnyError(format!("Failed to find an ip in the catalog")))?;
                }
            // use the local IP if the ip spec was not provided
            } else {
                target
            }
        // use the local IP if a path was supplied
        } else {
            target
        };
        // println!("{:?},", target);
        let target = match target {
            Some(t) => t,
            None => return Err(AnyError(format!("Failed to find an ip to install")))?,
        };

        // println!("{:?}", target.get_uuid());

        // verify the ip is not already taken in the cache
        {
            let check = Ip::compute_checksum(&target.get_root());

            let mut matches = HashSet::new();
            let slot_name = CacheSlot::new(
                target.get_man().get_ip().get_name(),
                target.get_man().get_ip().get_version(),
                &check,
            );

            if let Ok(mut rd) = read_dir(c.get_cache_path()) {
                let pat = format!(
                    "{}-{}",
                    target.get_man().get_ip().get_name(),
                    target.get_man().get_ip().get_version()
                );

                // upon force, will remove all installations (even dynamics)
                while let Some(d) = rd.next() {
                    if let Ok(p) = d {
                        if p.file_name().into_string().unwrap().starts_with(&pat) == true {
                            if self.force == true {
                                std::fs::remove_dir_all(&p.path())?;
                            } else {
                                matches.insert(p.file_name().into_string().unwrap());
                            }
                        }
                    }
                }
            }

            if self.force == false {
                if matches.contains(&slot_name.to_string()) == true {
                    println!(
                        "info: ip {} is already installed",
                        target.get_man().get_ip().into_ip_spec()
                    );
                    return Ok(());
                } else if matches.len() > 0 {
                    return Err(AnyError(format!(
                        "ip {} already exists in cache under different checksum",
                        target.get_man().get_ip().into_ip_spec()
                    )))?;
                }
            }
        }

        let download_slot = catalog.get_downloaded_slot(
            target.get_man().get_ip().get_name(),
            target.get_man().get_ip().get_version(),
        );
        // println!("{:?}", download_slot);

        // remove the download slot on purpose to replace it when using --force
        if download_slot.is_some()
            && catalog.is_downloaded_slot(&download_slot.as_ref().unwrap())
            && self.force == true
        {
            std::fs::remove_file(
                &c.get_downloads_path()
                    .join(&download_slot.as_ref().unwrap().as_ref()),
            )?;
        }

        // @MARK: check for when there are multiple uuids that could potentially be for this ip

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if target.can_use_lock() == true && self.force == false {
            println!("info: {}", "reading dependencies from lockfile ...");
            let env = Environment::new()
                // read config.toml for setting any env variables
                .from_config(c.get_config())?;
            let vtable = VariableTable::new().load_environment(&env)?;

            let le = LockEntry::from((&target, true));

            let lf = target.get_lock().keep_dev_dep_entries(&target, self.all);

            plan::download_missing_deps(
                vtable,
                &lf,
                &le,
                &catalog,
                &c.get_config().get_protocols(),
            )?;
            // recollect the queued items to update the catalog
            catalog = catalog.downloads(c.get_downloads_path())?;

            plan::install_missing_deps(&lf, &le, &catalog)?;
            // recollect the installations and queued items to update the catalog
            catalog = catalog.installations(c.get_cache_path())?;
        }

        // @MARK: may be an issue and should error if trying to install with an out-of-date lockfile
        // generate lock file if it is missing or out of date
        if target.lock_exists() == false || target.can_use_lock() == false {
            // build entire ip graph and resolve with dynamic symbol transformation
            let ip_graph = algo::compute_final_ip_graph(&target, &catalog, &c.get_languages())?;
            Plan::write_lockfile(&target, &ip_graph, true)?;
        }

        // move the IP to the downloads folder if not already there
        if download_slot.is_none()
            || catalog.is_downloaded_slot(&download_slot.as_ref().unwrap()) == false
        {
            Download::move_to_download_dir(
                &target.get_root(),
                c.get_downloads_path(),
                &target
                    .get_man()
                    .get_ip()
                    .into_ip_spec()
                    .to_partial_ip_spec(),
            )?;
        }

        // install the top-level target
        self.run(&target, &catalog)
    }
}

impl Install {
    fn download_target_from_url(&self, c: &Context, url: &str) -> Result<(), Fault> {
        let env = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?;
        let mut vtable = VariableTable::new().load_environment(&env)?;
        env.initialize();

        let protocols: ProtocolMap = c.get_config().get_protocols();

        let target_source = Source::new()
            .url(url.to_string())
            .protocol(self.protocol.clone())
            .tag(self.tag.clone());

        // fetch from the internet
        Download::download(
            &mut vtable,
            &self.ip.as_ref().unwrap(),
            &target_source,
            None,
            c.get_downloads_path(),
            &protocols,
            self.verbose,
            self.force,
        )?;

        Ok(())
    }

    pub fn is_checksum_good(root: &PathBuf) -> bool {
        // verify the checksum
        if let Some(sha) = Ip::read_checksum_proof(&root) {
            // make sure the sums match expected
            sha == Ip::compute_checksum(&root)
        // failing to compute a checksum
        } else {
            false
        }
    }

    /// Installs the `ip` with particular partial `version` to the `cache_root`.
    /// It will reinstall if it finds the original installation has a mismatching checksum.
    ///
    /// Returns `true` if the IP was successfully installed and `false` if it already existed.
    pub fn install(src: &Ip, cache_root: &std::path::PathBuf, force: bool) -> Result<bool, Fault> {
        // temporary destination to move files for processing and manipulation
        let dest = tempfile::tempdir()?.into_path();
        filesystem::copy(src.get_root(), &dest, true, Some(src.get_files_to_keep()))?;

        // lookup the package name in the index to see if the UUIDs match
        // verify the version for this package is not already logged

        // @note: a package's index file contains all metadata for all versions known to orbit
        // @note: ability to link various index directories (essentially vendors)
        // @note: also want to store zipped archives of installs in the "vault" for quicker retrieval

        // @todo: listing all units

        // @todo: store a LUT for unit names to the correct file to read when computing "get" command

        // @todo: getting the size of the entire directory

        // access the name and version
        let version = src.get_man().get_ip().get_version();
        let target = src.get_man().get_ip().get_name();
        let ip_spec = src.get_man().get_ip().into_ip_spec();
        println!("info: installing ip {} ...", &ip_spec);

        // perform sha256 on the temporary cloned directory
        let checksum = Ip::compute_checksum(&dest);
        // println!("checksum: {}", checksum);

        // use checksum to create new directory slot
        let cache_slot_name = CacheSlot::new(target, &version, &checksum);
        let cache_slot = cache_root.join(&cache_slot_name.to_string());
        // check if the slot is occupied in the cache
        if cache_slot.exists() == true {
            // check if we should proceed with force regardless if the installation is valid
            if force == true {
                std::fs::remove_dir_all(&cache_slot)?;
            } else {
                // ip is already installed
                if Self::is_checksum_good(&cache_slot) == true {
                    // clean up the temporary directory ourself
                    fs::remove_dir_all(dest)?;
                    return Ok(false);
                } else {
                    println!("info: reinstalling ip {} due to bad checksum ...", ip_spec);

                    // blow directory up for re-install
                    std::fs::remove_dir_all(&cache_slot)?;
                }
            }
        }
        // copy contents into cache slot from temporary destination
        crate::util::filesystem::copy(&dest, &cache_slot, false, Some(src.get_files_to_keep()))?;

        // clean up the temporary directory ourself
        fs::remove_dir_all(dest)?;

        // write the checksum to the directory (this file is excluded from auditing)
        std::fs::write(
            &cache_slot.join(ORBIT_SUM_FILE),
            checksum.to_string().as_bytes(),
        )?;

        Ok(true)
    }

    fn run(&self, target: &Ip, catalog: &Catalog) -> Result<(), Fault> {
        let result = Self::install(&target, &catalog.get_cache_path(), self.force)?;

        if result == false {
            println!(
                "info: ip {} is already installed",
                target.get_man().get_ip().into_ip_spec()
            );
        }

        Ok(())
        // store results from expensive computations into specific orbit files

        // print download list for top-level package
        // if self.compile == true {
        //     for s in Self::compile_download_list(ip.get_lock(), &catalog, false) {
        //         println!("{}", s);
        //     }
        //     return Ok(())
        // }

        // _pkg.get_lock().save_to_disk(&_pkg.get_root())?;
        // todo!();

        // @todo: check lockfile to process installing any IP that may be already downloaded to the queue

        // verify each requirement for the IP is also installed (o.w. install)

        // if let Some(lock) = man.get_lockfile() {
        //     Self::install_from_lock_file(&self, &lock, &catalog)?;
        // }
        // if the lockfile is invalid, then it will only install the current request and zero dependencies
    }
}

// # install from online using custom protocol
// orbit install toolbox:1.0.1 --url https://github.com/cdotrus/toolbox.git --protocol git-op

// # install from local path
// orbit install hamming:1.0.0 --path .

// # install from online using default protocol
// orbit install --url https://github.com/cdotrus/toolbox/archive/refs/tags/1.0.1.zip

// # install from queue
// orbit install toolbox:1.0.1

// error if multiple packages are located in a downloaded area (then they must supply a ip spec)
