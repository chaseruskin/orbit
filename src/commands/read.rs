use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;
use tempfile::TempPath;
use tempfile::tempfile;

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
        let editor = self.editor.as_ref().unwrap_or(&String::new()).to_owned();

        // determine the destination
        let dest = c.get_home_path().join("tmp");

        // must be in an IP if omitting the pkgid
        if self.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }

            self.run(&editor, &IpManifest::from_path(c.get_ip_path().unwrap())?, &dest) 
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
            let _ip = status.get(v, true);

            Ok(())
        }
    }
}

impl Read {
    fn run(&self, editor: &str, manifest: &IpManifest, dest: &PathBuf) -> Result<(), Fault> {
        Self::read(&self.unit, &manifest, &editor, &dest)
    }

    fn read(unit: &Identifier, ip: &IpManifest, editor: &str, dest: &PathBuf) -> Result<(), Fault> {
        // find the unit
        let units = ip.collect_units(true)?;
        // create a temporary file
        let path = TempPath::from_path("./tmp.txt");
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(&path)?;
        file.write_all("go gators".as_bytes())?;
        file.sync_all()?;
        std::process::Command::new(r#"C:\Users\cruskin\AppData\Local\Programs\Microsoft VS Code\code"#)
            .arg(path.as_os_str().to_str().unwrap())
            .spawn()?;
        todo!()
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