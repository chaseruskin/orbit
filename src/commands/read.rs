use std::io::BufReader;
use std::io::Write;
use std::io::Read as ReadTrait;
use std::path::PathBuf;
use colored::Colorize;

use clif::cmd::{FromCli, Command};
use crate::OrbitResult;
use crate::core::catalog::Catalog;
use crate::core::catalog::CatalogError;
use crate::core::lang::lexer::Position;
use crate::core::manifest::IpManifest;
use crate::core::pkgid::PkgId;
use crate::core::version::AnyVersion;
use crate::core::lang::vhdl::token::Identifier;
use clif::Cli;
use clif::arg::{Flag, Positional, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::sha256::compute_sha256;
use crate::util::filesystem::Standardize;

use super::edit::Edit;
use super::edit::EditMode;
use super::v2::get::GetError;

#[derive(Debug, PartialEq)]
pub struct Read {
    unit: Identifier,
    ip: Option<PkgId>,
    version: Option<AnyVersion>,
    editor: Option<String>,
    location: bool,
    mode: EditMode,
    no_clean: bool,
}

impl FromCli for Read {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Read {
            version: cli.check_option(Optional::new("variant").switch('v').value("version"))?,
            ip: cli.check_option(Optional::new("ip").value("pkgid"))?,
            editor: cli.check_option(Optional::new("editor"))?,
            mode: cli.check_option(Optional::new("mode"))?.unwrap_or(EditMode::Open),
            location: cli.check_flag(Flag::new("location"))?,
            no_clean: cli.check_flag(Flag::new("no-clean"))?,
            unit: cli.require_positional(Positional::new("unit"))?,
        });
        command
    }
}

impl Command<Context> for Read {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // determine the text-editor
        let editor = Edit::configure_editor(&self.editor, c.get_config())?;

        // determine the destination
        let dest = c.get_home_path().join(TMP_DIR);
        
        // attempt to clean the tmp directory unless --no-clean
        if dest.exists() == true && self.no_clean == false {
            // do not error if this procedure fails
            match std::fs::remove_dir_all(&dest) {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        // must be in an IP if omitting the pkgid
        if self.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }

            self.run(&editor, &IpManifest::from_path(c.get_ip_path().unwrap())?, None) 
        // checking external IP
        } else {
            // gather the catalog (all manifests)
            let catalog = Catalog::new()
                .store(c.get_store_path())
                .development(c.get_development_path().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?;
            let ids = catalog.inner().keys().map(|f| { f }).collect();
            let target = crate::core::ip::find_ip(&self.ip.as_ref().unwrap(), ids)?;
            
            // find all manifests and prioritize installed manifests over others but to help with errors/confusion
            let status = catalog.inner().get(&target).unwrap();

            // determine version to grab
            let v = self.version.as_ref().unwrap_or(&AnyVersion::Latest);

            if let Some(ip) = status.get(v, true) {
                self.run(&editor, &ip, Some(&dest))
            } else {
                if status.get(v, false).is_some() == true {
                    Err(CatalogError::SuggestInstall(target, v.clone()))?
                } else {
                    Err(CatalogError::NoVersionForIp(target, v.clone()))?
                }
            }
        }
    }
}

impl Read {
    fn run(&self, editor: &str, manifest: &IpManifest, dest: Option<&PathBuf>) -> Result<(), Fault> {
        let (path, loc) = Self::read(&self.unit, &manifest, dest)?;

        let path = { if self.location == true { PathBuf::from({ let mut p = path.as_os_str().to_os_string(); p.push(&loc.to_string()); p }) } else { path }};

        match &self.mode {
            EditMode::Path => {
                println!("{}", PathBuf::standardize(path).display());
            },
            EditMode::Open => {
                Edit::invoke(editor, &path)?;
            },
        };
        Ok(())
    }

    /// Finds the filepath and file position for the provided primary design unit `unit`
    /// under the project `ip`.
    /// 
    /// If `dest` contains a value, it will create a new directory at `dest` and copy
    /// the file to be read-only. If it is set to `None`, then it will open the
    /// file it is referencing (no copy). 
    fn read(unit: &Identifier, ip: &IpManifest, dest: Option<&PathBuf>) -> Result<(PathBuf, Position), Fault> {
        // find the unit
        let units = ip.collect_units(true)?;

        // get the file data for the primary design unit
        let (source, position) = match units.get_key_value(unit) {
            Some((_, unit)) => (unit.get_unit().get_source_code_file(), unit.get_unit().get_symbol().unwrap().get_position().clone()),
            None => return Err(GetError::SuggestShow(
                GetError::EntityNotFound(unit.clone(), ip.get_pkgid().get_name().clone(), ip.get_version().clone()).to_string(), 
                ip.get_pkgid().get_name().clone(), 
                ip.get_version().clone()))?
        };

        let (checksum, bytes) = {
            // open the file and create a checksum
            let mut bytes = Vec::new();
            let file = std::fs::File::open(source)?;
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut bytes)?;
            (compute_sha256(&bytes), bytes)
        };

        let src = PathBuf::from(source);

        match dest {
            // return direct reference if no `dest` (within current ip)
            None => return Ok((src, position)),
            Some(dest) => {
                // create new file under checksum directory
                let dest = dest.join(&checksum.to_string().get(0..10).unwrap());
                std::fs::create_dir_all(&dest)?;

                // add filename to destination path
                let dest = dest.join(src.file_name().unwrap());

                // try to remove file if it exists
                if dest.exists() == false || std::fs::remove_file(&dest).is_ok() {
                    // create and write a temporary file
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&dest)?;
                    file.write(&bytes)?;
                    file.flush()?;

                    // set to read-only
                    let mut perms = file.metadata()?.permissions();
                    perms.set_readonly(true);
                    file.set_permissions(perms)?;
                }
                Ok((dest, position))
            },
        }
    }
}

const TMP_DIR: &str = "tmp";

const HELP: &str = "\
Inspect hdl design unit source code.

Usage:
    orbit read [options] <unit>

Args:
    <unit>                  primary design unit identifier

Options:            
    --ip <pkgid>            ip to reference the unit from
    --variant, -v <version> ip version to use
    --editor <editor>       the command to invoke a text-editor
    --location              append the :line:col to the filepath
    --mode <mode>           select how to read: 'open' or 'path'
    --no-clean              do not delete previous files read

Use 'orbit help read' to learn more about the command.
";