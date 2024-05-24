use super::new::New;
use crate::commands::helps::init;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::manifest::{Manifest, IP_MANIFEST_FILE};
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::AnyError;
use crate::util::filesystem::Standardize;
use crate::util::filesystem::{self, ORBIT_IGNORE_FILE};
use std::io::Write;
use std::path::PathBuf;

use cliproc::{cli, proc};
use cliproc::{Cli, Flag, Help, Optional, Positional, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Init {
    force: bool,
    name: Option<PkgPart>,
    path: PathBuf,
}

impl Subcommand<Context> for Init {
    fn construct<'c>(cli: &'c mut Cli) -> cli::Result<Self> {
        cli.check_help(Help::default().text(init::HELP))?;
        Ok(Self {
            force: cli.check_flag(Flag::new("force"))?,
            name: cli.check_option(Optional::new("name"))?,
            path: cli
                .check_positional(Positional::new("path"))?
                .unwrap_or(PathBuf::from(".")),
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // @todo: verify the pkgid is not taken

        // @todo: refactor due to heavy overlap with 'new' command

        // resolve any relative path
        let dest = filesystem::full_normal(&self.path);
        // verify we are not already in an ip directory
        {
            if let Some(p) = Context::find_ip_path(&dest) {
                // @todo: write error
                panic!("An ip already exists at path {:?}", p)
            }
        }

        let ip_name = New::extract_name(self.name.as_ref(), &dest)?;

        self.create_ip(&ip_name, &c.get_build_dir())
    }
}

impl Init {
    /// Initializes a project at an exising path.
    fn create_ip(&self, ip: &PkgPart, build_dir: &str) -> AnyResult<()> {
        // verify the directory already exists
        if self.path.is_dir() == false || self.path.exists() == false {
            return Err(Box::new(AnyError(format!(
                "The path {:?} is not an existing directory",
                PathBuf::standardize(self.path.clone())
            ))));
        }

        // create the file directly nested within the destination path
        let manifest_path = {
            let mut p = self.path.clone();
            p.push(IP_MANIFEST_FILE);
            p
        };

        let ignore_path = {
            let mut p = self.path.clone();
            p.push(ORBIT_IGNORE_FILE);
            p
        };

        let mut manifest = std::fs::File::create(&manifest_path)?;
        let mut ignore = std::fs::File::create(&ignore_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip).as_bytes())?;
        ignore.write_all(Ip::write_default_ignore_file(build_dir).as_bytes())?;
        Ok(())
    }
}
