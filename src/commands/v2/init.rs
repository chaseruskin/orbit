use clif::cmd::{FromCli, Command};
use clif::Cli;
use clif::arg::{Optional, Flag};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgPart;
use crate::OrbitResult;
use crate::util::filesystem::Standardize;
use std::path::PathBuf;
use std::io::Write;
use crate::core::manifest2::Manifest;
use super::super::orbit::AnyResult;
use super::new::New;

#[derive(Debug, PartialEq)]
pub struct Init {
    force: bool,
    path: PathBuf,
    name: Option<PkgPart>,
}

impl FromCli for Init {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Self {
            force: cli.check_flag(Flag::new("force"))?,
            path: cli.check_option(Optional::new("path"))?.unwrap_or(PathBuf::from(".")),
            name: cli.check_option(Optional::new("ip"))?,
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
        let dest = PathBuf::standardize(self.path.clone());
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
            return Err(Box::new(AnyError(format!("the path {:?} is not an already existing directory", PathBuf::standardize(self.path.clone())))));
        }

        // create the file directly nested within the destination path
        let manifest_path = {
            let mut p = self.path.clone();
            p.push("Orbit.toml");
            p
        };

        let mut manifest = std::fs::File::create(&manifest_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip).as_bytes())?;
        Ok(())
    }

}

const HELP: &str = "\
Initialize a new ip from an existing project.

Usage:
    orbit init [options]

Options:
    --path <path>       destination path to initialize (default: '.')
    --ip <ip>           the pkgid to label the existing project
    --force             overwrite any existing manifest with a new one

Use 'orbit help init' to learn more about the command.
";