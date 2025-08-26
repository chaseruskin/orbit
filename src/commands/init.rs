//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use super::lock::Lock;
use super::new::New;
use crate::commands::helps::init;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::manifest::{Manifest, PROJECT_MANIFEST_FILE};
use crate::core::name::Name;
use crate::core::project::Project;
use crate::core::uuid::Uuid;
use crate::error::{Error, LastError};
use crate::util::anyerror::AnyError;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use crate::*;
use std::io::Write;
use std::path::PathBuf;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Init {
    name: Option<Name>,
    library: Option<Identifier>,
    path: PathBuf,
    uuid: bool,
}

impl Subcommand<Context> for Init {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(init::HELP))?;
        Ok(Self {
            uuid: cli.check(Arg::flag("uuid"))?,
            name: cli.get(Arg::option("name"))?,
            library: cli.get(Arg::option("lib"))?,
            path: cli
                .get(Arg::positional("path"))?
                .unwrap_or(PathBuf::from(".")),
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        if self.uuid == true {
            println!("{}", Uuid::new().encode());
            return Ok(());
        }

        // resolve any relative path
        let dest = filesystem::full_normal(&self.path);

        // verify this current path does have an existing project
        if let Some(c_path) = c.get_project_path() {
            if c_path == &dest {
                return Err(Error::IpExistsAtPath(dest))?;
            }
        }

        let ip_name = New::extract_name(self.name.as_ref(), &dest)?;

        match self.create_ip(&ip_name, c.are_units_private_by_default()) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::FailedToInitIp(LastError(e.to_string())))?,
        }
    }
}

impl Init {
    /// Initializes a project at an exising path.
    fn create_ip(&self, ip: &Name, priv_by_def: bool) -> AnyResult<()> {
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
            p.push(PROJECT_MANIFEST_FILE);
            p
        };

        let lib_str = match &self.library {
            Some(s) => Some(s.to_string()),
            None => None,
        };

        // write the manifest
        let mut manifest = std::fs::File::create(&manifest_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip, &lib_str).as_bytes())?;

        // write the lockfile
        let local_ip = Project::load(self.path.clone(), true, false)?;
        Lock::write_new_lockfile(&local_ip, true, priv_by_def)?;

        info!(
            "initialized project \"{}\"",
            local_ip.get_man().get_project().get_name()
        );

        // display the help message
        info!("{}", Manifest::write_manifest_ref_help());

        Ok(())
    }
}
