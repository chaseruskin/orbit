use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::core::manifest;
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
        let manifests = manifest::find_dev_manifests(c.get_development_path().as_ref().unwrap())?;
        // determine editor
        let sel_editor = match &self.editor {
            // first check if cli arg is empty
            Some(e) => e.to_owned(),
            None => {
                if let Ok(val) = std::env::var("EDITOR") {
                    val
                } else {
                    // try the config.toml
                    if c.get_config().contains_table("core") && c.get_config().contains_key("editor") {
                        if let Some(e) = c.get_config()["core"]["editor"].as_str() {
                            e.to_owned()
                        } else {
                            return Err(Box::new(AnyError("no editor detected".to_owned())))
                        }
                    } else {
                        return Err(Box::new(AnyError("no editor detected".to_owned())))
                    }
                }
            }
        };
        self.run(&manifests, &sel_editor)
    }
}

use crate::core::ip;

impl Edit {
    fn run(&self, manifests: &[manifest::Manifest], editor: &str) -> Result<(), Box<dyn std::error::Error>> {
        // find the full ip name among the manifests to get the path
        let result = ip::find_ip(&self.ip, &manifests)?;
        let mut root = result.get_path().to_owned();
        root.pop();
        // perform the process
        let _ = std::process::Command::new(editor)
            .args(&[root.display().to_string()])
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