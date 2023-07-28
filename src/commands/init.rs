use super::new::New;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::manifest::{Manifest, IP_MANIFEST_FILE};
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::AnyError;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use crate::OrbitResult;
use clif::arg::{Flag, Optional, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use std::io::Write;
use std::path::PathBuf;
use crate::commands::helps::init;

#[derive(Debug, PartialEq)]
pub struct Init {
    force: bool,
    name: Option<PkgPart>,
    path: PathBuf,
}

impl FromCli for Init {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(init::HELP).ref_usage(2..4))?;
        let command = Ok(Self {
            force: cli.check_flag(Flag::new("force"))?,
            name: cli.check_option(Optional::new("name"))?,
            path: cli
                .check_positional(Positional::new("path"))?
                .unwrap_or(PathBuf::from(".")),
        });
        command
    }
}

impl Command<Context> for Init {
    type Status = OrbitResult;

    fn exec(&self, _c: &Context) -> Self::Status {
        // @todo: verify the pkgid is not taken

        // @todo: refactor due to heavy overlap with 'new' command

        // resolve any relative path
        let dest = filesystem::full_normal(&self.path);
        // verify we are not already in an ip directory
        {
            if let Some(p) = Context::find_ip_path(&dest) {
                // @todo: write error
                panic!("an ip already exists at path {:?}", p)
            }
        }

        let ip_name = New::extract_name(self.name.as_ref(), &dest)?;

        self.create_ip(&ip_name)
    }
}

impl Init {
    /// Initializes a project at an exising path.
    fn create_ip(&self, ip: &PkgPart) -> AnyResult<()> {
        // verify the directory already exists
        if self.path.is_dir() == false || self.path.exists() == false {
            return Err(Box::new(AnyError(format!(
                "The path {:?} is not an already existing directory",
                PathBuf::standardize(self.path.clone())
            ))));
        }

        // create the file directly nested within the destination path
        let manifest_path = {
            let mut p = self.path.clone();
            p.push(IP_MANIFEST_FILE);
            p
        };

        let mut manifest = std::fs::File::create(&manifest_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip).as_bytes())?;
        Ok(())
    }
}