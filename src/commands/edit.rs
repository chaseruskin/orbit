use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::{PkgId, PkgPart};
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
                    if let Some(e) = c.get_config()["core"]["editor"].as_str() {
                        e.to_owned()
                    } else {
                        return Err(Box::new(AnyError("no editor detected".to_owned())))
                    }
                }
            }
        };
        self.run(&manifests, &sel_editor)
    }
}

use crate::util::overdetsys;
use crate::core::manifest::Manifest;

/// Given a partial/full ip specification `ip_spec`, sift through the manifests
/// for a possible determined unique solution.
pub fn find_ip<'a>(ip_spec: &PkgId, manifests: &'a [manifest::Manifest]) -> Result<&'a Manifest, AnyError> {
    // try to find ip name
    let space: Vec<Vec<PkgPart>> = manifests.iter().map(|f| { f.as_pkgid().into_full_vec().unwrap() }).collect();
    let result = match overdetsys::solve(space, ip_spec.iter()) {
        Ok(r) => r,
        Err(e) => match e {
            overdetsys::OverDetSysError::NoSolution => Err(AnyError(format!("no ip as '{}' exists", ip_spec)))?,
            overdetsys::OverDetSysError::Ambiguous(set) => {
                // assemble error message
                let mut set = set.into_iter().map(|f| PkgId::from_vec(f) );
                let mut content = String::new();
                while let Some(s) = set.next() {
                    content.push_str(&format!("    {}\n", s.to_string()));
                }
                Err(AnyError(format!("ambiguous ip '{}' yields multiple solutions:\n{}", ip_spec, content)))?
            }
        }
    };

    let full_ip = PkgId::from_vec(result);
    // find the full ip name among the manifests to get the path
    Ok(manifests.iter().find(|f| { full_ip == f.as_pkgid() }).unwrap())
}

impl Edit {
    fn run(&self, manifests: &[manifest::Manifest], editor: &str) -> Result<(), Box<dyn std::error::Error>> {
        // find the full ip name among the manifests to get the path
        let result = find_ip(&self.ip, &manifests)?;
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