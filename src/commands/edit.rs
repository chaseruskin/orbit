use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Edit {
    editor: Option<String>,
    ip: PkgId,
}

impl FromCli for Edit {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Edit {
            editor: cli.check_option(Optional::new("editor"))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        });
        command
    }
}

impl Command for Edit {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // collect manifest from DEV_PATH
        let catalog = Catalog::new()
            .development(c.get_development_path().as_ref().unwrap())?;
        // determine editor
        let sel_editor = match &self.editor {
            // first check if cli arg is empty
            Some(e) => e.to_owned(),
            None => {
                if let Ok(val) = std::env::var("EDITOR") {
                    val
                } else {
                    // try the config.toml
                    match c.get_config().get_as_str("core", "editor")? {
                        Some(e) => e.to_owned(),
                        None => return Err(Box::new(AnyError("no editor detected".to_owned())))
                    }
                }
            }
        };
        self.run(&catalog, &sel_editor)
    }
}

use crate::core::ip;

impl Edit {
    fn run(&self, catalog: &Catalog, editor: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ids: Vec<&PkgId> = catalog.inner().keys().map(|f| f).collect();
        // find the full ip name among the manifests to get the path
        let result = ip::find_ip(&self.ip, ids)?;

        let ip = catalog.inner().get(&result).unwrap().get_dev().unwrap();
        // perform the process
        let _ = std::process::Command::new(editor)
            .args(&[ip.get_root().display().to_string()])
            .spawn()?;
    
        Ok(())
    }
}

const HELP: &str = "\
Open a text editor to develop an ip.

Usage:
    orbit edit [options] <ip>

Args:
    <ip>               the pkgid to find the ip under ORBIT_PATH

Options:
    --editor <cmd>     the command to call a text-editor

Use 'orbit help edit' to learn more about the command.
";