//
//  Copyright (C) 2022-2026  Chase Ruskin
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

use crate::commands::helps::clean;
use crate::core::context::Context;
use crate::core::project::Project;
use crate::error::{Error, Hint};
use crate::util::anyerror::AnyError;
use crate::*;
use std::fs;
use std::path::Path;

use cliproc::{Arg, Cli, Help, Subcommand};
use cliproc::{cli, proc, stage::*};

#[derive(Debug, PartialEq)]
pub struct Clean {
    /// Whether or not to clean just the documentation directory
    doc: bool,
    target: Option<String>,
    target_dir: Option<String>,
}

impl Subcommand<Context> for Clean {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(clean::HELP))?;
        Ok(Self {
            // flags
            doc: cli.check(Arg::flag("doc"))?,
            // options
            target: cli.get(Arg::option("target").value("name").switch('t'))?,
            target_dir: cli.get(Arg::option("target-dir").value("dir"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // arguments are mutually exclusive
        if self.doc == true && self.target.is_some() {
            return Err(Box::new(AnyError::from(format!("a target cannot also be selected when requesting the documentation to be removed").as_str())));
        }

        // verify running from an ip directory and enter ip's root directory
        c.jump_to_working_project()?;

        let current_project = Project::load(
            c.get_project_path().unwrap().to_path_buf(),
            true,
            true,
            false,
        )?;

        // determine the build directory based on cli priority
        let default_target_dir = c.get_target_dir();
        let target_dir = self.target_dir.as_ref().unwrap_or(&default_target_dir);

        // path where all targets are to be kept
        let target_path = current_project.get_root().join(target_dir);

        // try to get the target if one was requested (ignore errors)
        let sel_target = c.select_target(&self.target, false, true).unwrap_or(None);
        let sel_target = match sel_target {
            Some(t) => Some(t),
            None => c.select_target(&self.target, false, false).unwrap_or(None),
        };
        if sel_target.is_none() && self.target.is_some() {
            return Err(Box::new(Error::TargetNotFound(
                self.target.unwrap().to_string(),
                Hint::TargetsListAll,
            )));
        }

        let dir_to_remove = match sel_target {
            Some(t) => target_path.join(t.get_name()),
            None => match self.doc {
                true => target_path.join("doc"),
                false => target_path,
            },
        };

        // TODO: Need to add mutex/locking mechanism on this section of the code to ensure it has full control of this directory before removing

        if Path::exists(&dir_to_remove) == true {
            fs::remove_dir_all(&dir_to_remove)?;
            info!(
                "removed directory \"{}\"",
                crate::util::filesystem::into_std_str(dir_to_remove)
            );
        } else {
            info!(
                "directory \"{}\" is already clean",
                crate::util::filesystem::into_std_str(dir_to_remove)
            );
        }
        Ok(())
    }
}
