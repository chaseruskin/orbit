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

use crate::commands::helps::new;
use crate::commands::plan;
use crate::commands::plan::Plan;
use crate::core::blueprint::Scheme;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::project::Project;
use crate::core::target::Target;
use crate::error::{Error, LastError};
use crate::util::environment::Environment;
use cliproc::{Arg, Cli, Help, Subcommand};
use cliproc::{cli, proc, stage::*};

use crate::util::filesystem::LockZone;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;

#[derive(Debug, PartialEq)]
pub struct Analyze {
    /// Determine the format in which to print the blueprint.
    plan: Scheme,
    /// Only return the entries that belong to the current working project.
    local: bool,
    /// Force the analysis even if we have broken HDL.
    force: bool,
}

impl Subcommand<Context> for Analyze {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(new::HELP))?;
        Ok(Self {
            local: cli.check(Arg::flag("local"))?,
            force: cli.check(Arg::flag("force"))?,
            plan: cli
                .get(Arg::option("plan").value("format"))?
                .unwrap_or_default(),
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // Verify running from a project's directory and enter the project's root directory.
        c.jump_to_working_project()?;

        let current_project = Project::load(
            c.get_project_path().unwrap().to_path_buf(),
            true,
            true,
            false,
        )?;

        // Before we gather the catalog, request an "APPEND" action to the cache.
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // Gather the catalog and resolve any missing dependencies.
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;
        plan::resolve_missing_deps(c, &current_project, &mut catalog, self.force)?;
        let catalog = catalog;

        let target = Target::default();

        // Plan.
        let result = Plan::run(
            &current_project,
            None,
            &target,
            catalog,
            self.force,
            &None,
            &None,
            &None,
            &self.plan,
            true,
            true,
            false,
            self.local,
            Environment::new(),
            c.are_units_private_by_default(),
        );

        // Release our "APPEND" action to the cache.
        crate::util::filesystem::release_lock(&cache_ap_lock)?;

        match result {
            Ok(r) => {
                if let Some(blueprint) = r {
                    print!("{}", blueprint);
                }
            }
            Err(e) => Err(Error::TargetProcFailed(LastError(e.to_string())))?,
        }
        Ok(())
    }
}
