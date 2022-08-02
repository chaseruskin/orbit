use std::io::BufReader;
use std::io::Write;
use std::io::Read as ReadTrait;
use std::path::PathBuf;
use colored::Colorize;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::core::pkgid::PkgId;
use crate::core::version::AnyVersion;
use crate::core::vhdl::token::Identifier;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::sha256::compute_sha256;

#[derive(Debug, PartialEq)]
pub struct Read {
    unit: Identifier,
    ip: Option<PkgId>,
    version: Option<AnyVersion>,
    editor: Option<String>,
}

impl FromCli for Read {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Read {
            version: cli.check_option(Optional::new("variant").switch('v').value("version"))?,
            ip: cli.check_option(Optional::new("ip").value("pkgid"))?,
            unit: cli.require_positional(Positional::new("unit"))?,
            editor: cli.check_option(Optional::new("editor"))?,
        });
        command
    }
}

impl Command for Read {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // determine the text-editor

        let editor = match &self.editor {
            Some(e) => Some(e.as_ref()),
            None => c.get_config().get_as_str("core", "editor")?,
        };

        // verify we have a text editor
        if editor.is_none() == true {
            panic!("no editor selected to open file")
        }

        // determine the destination
        let dest = c.get_home_path().join("tmp");

        // must be in an IP if omitting the pkgid
        if self.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }

            self.run(&editor.unwrap(), &IpManifest::from_path(c.get_ip_path().unwrap())?, &dest) 
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
                self.run(&editor.unwrap(), &ip, &dest)
            } else {
                panic!("no usable ip")
            }
        }
    }
}

impl Read {
    fn run(&self, editor: &str, manifest: &IpManifest, dest: &PathBuf) -> Result<(), Fault> {
        Self::read(&self.unit, &manifest, &editor, &dest)
    }

    fn read(unit: &Identifier, ip: &IpManifest, _editor: &str, dest: &PathBuf) -> Result<(), Fault> {
        // find the unit
        let units = ip.collect_units(true)?;

        // get the file data for the primary design unit
        let (source, position) = match units.get_key_value(unit) {
            Some((_, unit)) => (unit.get_unit().get_source_code_file(), unit.get_unit().get_symbol().unwrap().get_position()),
            None => todo!()
        };

        let (checksum, bytes) = {
            // open the file and create a checksum
            let mut bytes = Vec::new();
            let file = std::fs::File::open(source)?;
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut bytes)?;
            (compute_sha256(&bytes), bytes)
        };

        // create new file under checksum directory
        let dest = dest.join(&checksum.to_string().get(0..10).unwrap());
        std::fs::create_dir_all(&dest)?;

        // add filename to destination path
        let dest = dest.join(PathBuf::from(source).file_name().unwrap());

        // create and write a file
        {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&dest)?;
            file.write(&bytes)?;
            file.flush()?;

            // // set to read-only
            // let mut perms = file.metadata()?.permissions();
            // perms.set_readonly(true);
            // file.set_permissions(perms)?;
        }

        println!("{}:{}", dest.display(), position);

        // std::process::Command::new(editor).arg(&format!("{}:{}", source, position)).spawn()?;

        // create a temporary file
        Ok(())
    }
}

const HELP: &str = "\
Inspect hdl design unit source code.

Usage:
    orbit read [options] <unit>

Options:
    <unit>                  the pkgid to find the ip under ORBIT_PATH
    --ip <pkgid>            ip to reference the unit from
    --variant, -v <version> state of ip to checkout
    --editor <editor>       the command to invoke a text-editor

Use 'orbit help read' to learn more about the command.
";