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

use super::plan::{self, Plan};
use crate::commands::download::Download;
use crate::commands::helps::lock;
use crate::core::algo;
use crate::core::catalog::Catalog;
use crate::core::catalog::PkgName;
use crate::core::context::Context;
use crate::core::lockfile::LockEntry;
use crate::core::name::Name;
use crate::core::project::Project;
use crate::core::swap::StrSwapTable;
use crate::core::uuid::Uuid;
use crate::core::version::AnyVersion;
use crate::core::version::PartialVersion;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;
use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Lock {
    force: bool,
}

impl Subcommand<Context> for Lock {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(lock::HELP))?;
        let command = Ok(Lock {
            // flags
            force: cli.check(Arg::flag("force"))?,
        });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        // check that user is in a project directory
        c.jump_to_working_project()?;

        let force_apply_new_uuid = self.force;

        // store the working ip struct
        let working_ip = Project::load(
            c.get_project_path().unwrap().clone(),
            true,
            force_apply_new_uuid,
        )?;

        // before we gather the catalog, request an "APPEND" action to the cache
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // assemble the catalog
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;

        // update lock file if manifest changed (will need to download and install)
        if working_ip.can_use_lock(&catalog) == false {
            crate::info!("synchronizing lockfile with manifest...");
            synchronize_state_with_manifest(&c, &working_ip, &mut catalog)?;
        }

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if working_ip.can_use_lock(&catalog) == true && self.force == false {
            synchronize_state_with_lockfile(&c, &working_ip, &mut catalog)?;
        }

        let result = Self::run(
            &working_ip,
            &catalog,
            self.force,
            c.are_units_private_by_default(),
        );
        // release our "APPEND" action to the cache
        crate::util::filesystem::release_lock(&cache_ap_lock)?;
        result
    }
}

/// Synchronizes the state of the catalog system based on the state of the lockfile.
///
/// This function assumes the lockfile is up-to-date with the manifest.
pub fn synchronize_state_with_lockfile(
    c: &Context,
    working_ip: &Project,
    catalog: &mut Catalog,
) -> Result<(), Fault> {
    let le: LockEntry = LockEntry::from((working_ip, true));
    let lf = working_ip.get_lock();

    let env = Environment::new()
        // read config.toml for setting any env variables
        .from_config(c.get_config())?;
    let vtable = StrSwapTable::new().load_environment(&env)?;

    plan::download_missing_deps(
        vtable,
        &lf,
        &le,
        &catalog,
        c.get_default_protocol(),
        &c.get_config().get_protocols(),
    )?;
    // recollect the downloaded items to update the catalog for installations
    catalog.refresh_downloads()?;

    plan::install_missing_deps(&lf, &le, &catalog)?;
    // recollect the installations to update the catalog for dependency graphing
    catalog.refresh_installations()?;
    Ok(())
}

type PrjId = (Name, Option<Uuid>, PartialVersion);

/// Synchronizes the state of the catalog system based on the state of the manifest.
///
/// This function assumes the lockfile is out of date (stale) in comparison to the manifest. It will
/// also automatically update the lockfile.
pub fn synchronize_state_with_manifest(
    c: &Context,
    working_ip: &Project,
    catalog: &mut Catalog,
) -> Result<(), Fault> {
    // Re-look at all dependencies
    let env = Environment::new()
        // read config.toml for setting any env variables
        .from_config(c.get_config())?;
    let mut vtable = StrSwapTable::new().load_environment(&env)?;
    env.initialize();

    let mut reqs: Vec<PrjId> = working_ip
        .get_man()
        .get_deps_list(true, true)
        .into_iter()
        .map(|(n, d)| {
            (
                n.clone(),
                match d.as_uuid() {
                    Some(u) => Some(u.clone()),
                    None => None,
                },
                d.get_version().clone(),
            )
        })
        .collect();

    let mut prjs_to_download = Vec::new();
    let mut prjs_to_install = Vec::new();

    // iterate through all dependencies (and add on as we go if we find more dependencies of dependencies...)
    while let Some((name, uuid, ver)) = reqs.pop() {
        // try to look up this project in the catalog
        let pkg_name = PkgName::new(&name, uuid.as_ref());
        match catalog.translate_name(&pkg_name)? {
            // check what the current status is of the project in the catalog
            Some(status) => {
                let any_ver = AnyVersion::Specific(ver.clone());
                // Take no action if the project is alread installed
                if let Some(_) = status.get_install(&any_ver) {
                    continue;
                // Install the missing project
                } else if let Some(dep_prj) = status.get_download(&any_ver) {
                    // perform extra work if the Ip is virtual (from downloads)
                    if let Some(bytes) = dep_prj.get_mapping().as_bytes() {
                        prjs_to_install.push((name, uuid, ver, bytes.clone()));
                    } else {
                        panic!("trying to download from non virtual path")
                    }
                    // add this dependency's requirements to the list of ones to check
                    let mut dep_reqs: Vec<(Name, Option<Uuid>, PartialVersion)> = dep_prj
                        .get_man()
                        .get_deps_list(false, true)
                        .into_iter()
                        .map(|(n, d)| {
                            (
                                n.clone(),
                                match d.as_uuid() {
                                    Some(u) => Some(u.clone()),
                                    None => None,
                                },
                                d.get_version().clone(),
                            )
                        })
                        .collect();
                    dep_reqs.extend(reqs.into_iter());
                    reqs = dep_reqs;
                // Download and install the missing project
                } else if let Some(dep_prj) = status.get_available(&any_ver) {
                    // prjs_to_install.push(dep_prj);
                    match dep_prj.get_man().get_project().get_source() {
                        Some(src) => {
                            prjs_to_download.push((
                                name,
                                uuid,
                                ver,
                                src.clone(),
                                dep_prj
                                    .get_man()
                                    .get_project()
                                    .into_project_id_spec()
                                    .to_partial_project_id_spec(),
                            ));
                        }
                        None => {
                            panic!("missing repository field for a project in a channel: please open a bug report");
                        }
                    }
                    // add this dependency's requirements to the list of ones to check
                    let mut dep_reqs: Vec<(Name, Option<Uuid>, PartialVersion)> = dep_prj
                        .get_man()
                        .get_deps_list(false, true)
                        .into_iter()
                        .map(|(n, d)| {
                            (
                                n.clone(),
                                match d.as_uuid() {
                                    Some(u) => Some(u.clone()),
                                    None => None,
                                },
                                d.get_version().clone(),
                            )
                        })
                        .collect();
                    dep_reqs.extend(reqs.into_iter());
                    reqs = dep_reqs;
                }
            }
            None => {
                continue;
            }
        }
    }

    // download all projects missing at the archive level
    for (name, uuid, ver, repo, spec) in prjs_to_download {
        let pkg_name = PkgName::new(&name, uuid.as_ref());
        match catalog.translate_name(&pkg_name)? {
            Some(stat) => {
                let any_ver = AnyVersion::Specific(ver.clone());
                if stat.get_download(&any_ver).is_none() {
                    let (_, bytes) = Download::download(
                        &mut vtable,
                        Some(&spec),
                        &repo,
                        catalog.get_downloads_path(),
                        c.get_default_protocol(),
                        &c.get_config().get_protocols(),
                        true,
                    )?;
                    // add this project to the list of projects to install
                    prjs_to_install.push((name, uuid, ver, bytes));
                }
            }
            None => {
                continue;
            }
        }
        // recollect the downloaded items to update the catalog for installations
        catalog.refresh_downloads()?;
    }

    // install all projects missing at the cache level
    for (name, uuid, ver, bytes) in prjs_to_install {
        let pkg_name = PkgName::new(&name, uuid.as_ref());
        match catalog.translate_name(&pkg_name)? {
            Some(stat) => {
                let any_ver = AnyVersion::Specific(ver.clone());
                if stat.get_install(&any_ver).is_none() {
                    plan::install_ip_from_downloads(&bytes, &catalog, true)?;
                }
            }
            None => {
                continue;
            }
        }
        // recollect the installations to update the catalog for dependency graphing
        catalog.refresh_installations()?;
    }

    update_lockfile(
        &working_ip,
        &catalog,
        true,
        c.are_units_private_by_default(),
    )?;
    Ok(())
}

pub fn update_lockfile(
    local_prj: &Project,
    catalog: &Catalog,
    force: bool,
    priv_by_def: bool,
) -> Result<(), Fault> {
    let prj_graph = algo::compute_final_project_graph(&local_prj, Some(&catalog), priv_by_def)?;
    Plan::write_lockfile(&local_prj, &prj_graph, force, true, &catalog)?;
    Ok(())
}

impl Lock {
    /// Performs the backend logic for creating a blueprint file (planning a design).
    pub fn run(
        working_ip: &Project,
        catalog: &Catalog,
        force: bool,
        priv_by_def: bool,
    ) -> Result<(), Fault> {
        if working_ip.can_use_lock(catalog) == true {
            update_lockfile(working_ip, catalog, force, priv_by_def)?;
        }
        Ok(())
    }

    /// Writes a lockfile for a newly created ip (one that either was made with `new` or `init`).
    pub fn write_new_lockfile(
        local_ip: &Project,
        warn: bool,
        priv_by_def: bool,
    ) -> Result<(), Fault> {
        // build entire ip graph and resolve with dynamic symbol transformation
        let catalog = Catalog::new();
        let ip_graph =
            match algo::compute_final_project_graph(&local_ip, Some(&catalog), priv_by_def) {
                Ok(g) => g,
                Err(e) => match warn {
                    true => {
                        crate::warn!("{}", e.1);
                        algo::minimal_graph_map(local_ip)
                    }
                    false => return Err(e)?,
                },
            };
        Plan::write_lockfile(&local_ip, &ip_graph, true, false, &catalog)?;
        Ok(())
    }
}
