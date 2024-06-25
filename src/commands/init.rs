use super::new::New;
use crate::commands::helps::init;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::manifest::{Manifest, IP_MANIFEST_FILE};
use crate::core::pkgid::PkgPart;
use crate::error::{Error, LastError};
use crate::util::anyerror::AnyError;
use crate::util::filesystem::Standardize;
use crate::util::filesystem::{self, ORBIT_IGNORE_FILE};
use std::io::Write;
use std::path::PathBuf;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Init {
    force: bool,
    name: Option<PkgPart>,
    path: PathBuf,
}

impl Subcommand<Context> for Init {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(init::HELP))?;
        Ok(Self {
            force: cli.check(Arg::flag("force"))?,
            name: cli.get(Arg::option("name"))?,
            path: cli
                .get(Arg::positional("path"))?
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
                return Err(Error::IpExistsAtPath(p))?;
            }
        }

        let ip_name = New::extract_name(self.name.as_ref(), &dest)?;

        match self.create_ip(&ip_name, &c.get_target_dir()) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::FailedToInitIp(LastError(e.to_string())))?,
        }
    }
}

impl Init {
    /// Initializes a project at an exising path.
    fn create_ip(&self, ip: &PkgPart, target_dir: &str) -> AnyResult<()> {
        // verify the directory already exists
        if self.path.is_dir() == false || self.path.exists() == false {
            return Err(Box::new(AnyError(format!(
                "the path {:?} is not an existing directory",
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
        ignore.write_all(Ip::write_default_ignore_file(target_dir).as_bytes())?;
        Ok(())
    }
}
