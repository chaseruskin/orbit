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

use cliproc::{cli, proc, stage::Memory, Arg, Cli, Help, Subcommand};

use crate::commands::helps::test;
use crate::core::blueprint::Scheme;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::fileset::Fileset;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::project::Project;
use crate::core::swap::StrSwapTable;
use crate::core::target::Process;
use crate::core::target::Target;
use crate::error::Error;
use crate::error::LastError;
use crate::util::environment::ORBIT;
use crate::util::environment::ORBIT_OUT_DIR;
use crate::util::environment::{EnvVar, Environment, ORBIT_TARGET_DIR};
use crate::util::filesystem::get_exe_path;
use crate::util::filesystem::into_std_str;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::Standardize;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;
use crate::util::filesystem::PRJ_CATALOG_SH_LOCK_NAME;
use crate::warn;
use std::path::PathBuf;

use super::plan::{self, Plan};

#[derive(Debug, PartialEq)]
pub struct Test {
    target: Option<String>,
    args: Vec<String>,
    list: bool,
    dirty: bool,
    target_dir: Option<String>,
    force: bool,
    all: bool,
    plan: Option<Scheme>,
    verbose: bool,
    dut: Option<Identifier>,
    command: Option<String>,
    filesets: Option<Vec<Fileset>>,
    bench: Option<Identifier>,
}

impl Subcommand<Context> for Test {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(test::HELP))?;
        Ok(Test {
            // Flags
            list: cli.check(Arg::flag("list").switch('l'))?,
            verbose: cli.check(Arg::flag("verbose"))?,
            force: cli.check(Arg::flag("force"))?,
            all: cli.check(Arg::flag("all"))?,
            dirty: cli.check(Arg::flag("keep").switch('k'))?,
            // Options
            dut: cli.get(Arg::option("dut").value("unit"))?,
            bench: cli.get(Arg::option("tb").value("unit"))?,
            plan: cli.get(Arg::option("plan").value("format"))?,
            target: cli.get(Arg::option("target").value("name").switch('t'))?,
            target_dir: cli.get(Arg::option("target-dir"))?,
            command: cli.get(Arg::option("command").value("path"))?,
            filesets: cli.get_all(Arg::option("fileset").value("key=glob"))?,
            // Remaining args
            args: cli.remainder()?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // locate the plugin
        let target = c.select_target(&self.target, self.list == false, false)?;

        // display plugin list and exit
        if self.list == true {
            // try to get the default target
            let def_target = c.select_target(&None, true, false).unwrap_or(None);
            match target {
                // display entire contents about the particular plugin
                Some(tar) => println!("{}", tar.to_string()),
                // display quick overview of all plugins
                None => print!(
                    "{}",
                    Target::list_targets(
                        &mut c
                            .get_config()
                            .get_targets(false)
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Target>>(),
                        def_target,
                    )
                ),
            }
            return Ok(());
        }

        let target = target.unwrap();

        // coordinate the plan
        let plan = target.coordinate_plan(&self.plan)?;

        // check that user is in an project directory
        c.jump_to_working_project()?;

        // create the ip manifest
        let current_project = Project::load(c.get_project_path().unwrap().clone(), true, false)?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // 2025-08-24 is this still relevant? -> probably no
        // see Install::install_from_lock_file

        // determine the build directory (command-line arg overrides configuration setting)
        let default_target_dir = c.get_target_dir();
        let target_dir = self.target_dir.as_ref().unwrap_or(&default_target_dir);
        let out_dir = target.get_name();

        // path where all targets are to be kept
        let target_path = current_project.get_root().join(target_dir);
        // path where the current selected target will be kept
        let output_path = current_project.get_root().join(target_dir).join(out_dir);

        // before we gather the catalog, request an "APPEND" action to the cache
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::PackageCache,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // gather the catalog and resolve any missing dependencies
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;
        let catalog = plan::resolve_missing_deps(c, &current_project, catalog, self.force)?;

        let envs = Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_project(&current_project)?
            .add(EnvVar::new().key(ORBIT_TARGET_DIR).value(target_dir))
            .add(
                EnvVar::new()
                    .key(ORBIT)
                    .value(&into_std_str(get_exe_path()?)),
            )
            .add(EnvVar::with(
                ORBIT_OUT_DIR,
                PathBuf::standardize(&output_path).to_str().unwrap(),
            ));

        // try to acquire a lock to only allow one orbit process access to the target output directory
        let (lockpath, _lockfd) = crate::util::filesystem::acquire_lock(
            &target_path,
            LockZone::OutputDir,
            Some(&target.get_name()),
            false,
        )?;

        // plan the target
        Plan::run(
            &current_project,
            target_dir,
            target,
            catalog,
            self.dirty == false,
            self.force,
            self.all,
            &self.bench,
            &self.dut,
            &self.filesets,
            &plan,
            true,
            true,
            envs,
            c.are_units_private_by_default(),
        )?;

        // before we read the source files in our process, request a shared "READ" action to the cache
        let (_cache_rd_path, cache_rd_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::PackageCache,
            Some(PRJ_CATALOG_SH_LOCK_NAME),
            crate::util::filesystem::OS_CAN_SHARE_LOCK,
        )?;

        // release our "APPEND" action to the cache
        crate::util::filesystem::release_lock(&cache_ap_lock)?;

        // prepare for build
        let envs = Environment::new().from_env_file(&output_path)?;

        let swap_table = StrSwapTable::new().load_environment(&envs)?;
        let target = target.clone().replace_vars_in_args(&swap_table);

        // run the command from the output path
        crate::info!("executing target {}", target.get_name().green());
        let result = target.execute(&self.command, &self.args, &output_path, envs.into_map());

        // release our "READ" action to the cache
        crate::util::filesystem::release_lock(&cache_rd_lock)?;

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
