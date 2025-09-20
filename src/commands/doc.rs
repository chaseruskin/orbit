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

use std::path::PathBuf;

use crate::commands::helps::doc;
use crate::core::context::Context;
use crate::core::project::Project;
use crate::error::Error;
use crate::error::LastError;
use crate::info;
use crate::util::filesystem::LockZone;
use crate::warn;

use cliproc::{cli, proc, stage::*};
use cliproc::{Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Doc {}

impl Subcommand<Context> for Doc {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(doc::HELP))?;
        let command = Ok(Doc {});
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        println!("{}", "preparing to generate documentation...");
        // must be ran from the local project

        // verify running from a project directory and enter project's root directory
        c.jump_to_working_project()?;
        let current_project =
            Project::load(c.get_project_path().unwrap().to_path_buf(), true, false)?;

        let target_name = "doc";

        // get the output path where we will generate the documentation
        let default_target_dir = c.get_target_dir();
        let target_dir = default_target_dir; // currently have no cli option to override
        let out_dir = target_name;

        // path where all targets are to be kept
        let target_path = current_project.get_root().join(&target_dir);
        // path where the documentation will be kept
        let output_path = current_project.get_root().join(&target_dir).join(out_dir);

        // try to acquire a lock to only allow one orbit process access to the target output directory
        let (lockpath, _lockfd) = crate::util::filesystem::acquire_lock(
            &target_path,
            LockZone::OutputDir,
            Some(&target_name),
            false,
        )?;

        // outputs to a target/doc folder
        let result = self.run(&current_project, &output_path);

        // unlock the target output directory
        match std::fs::remove_file(&lockpath) {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "{}",
                    Error::FileUnlockFailed(lockpath.clone(), e.to_string(),).to_string()
                );
            }
        }
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::TargetProcFailed(LastError(e.to_string())))?,
        }
    }
}

impl Doc {
    fn run(&self, _p: &Project, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "generating documentation...");

        info!(
            "documentation generated at: {}",
            crate::util::filesystem::into_std_str(output_path.to_path_buf())
        );
        Ok(())
    }
}
