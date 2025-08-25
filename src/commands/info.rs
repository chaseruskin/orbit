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

use crate::commands::helps::info;
use crate::core::cache::UnitCache;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::lang::LangUnit;
use crate::core::manifest::Manifest;
use crate::core::project::{PartialProjectIdSpec, Project};
use crate::core::version;
use crate::core::visibility::Visibility;
use crate::error::{Error, Hint};
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::PRJ_CACHE_SH_LOCK_NAME;
use colored::Colorize;
use std::cmp::Ordering;
use std::env::current_dir;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Info {
    // TODO: narrow the displayed version list with a range?
    versions: bool,
    units: bool,
    spec: Option<PartialProjectIdSpec>,
    all: bool,
    // TODO: view changelog?
    // TODO: view readme?
}

impl Subcommand<Context> for Info {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(info::HELP))?;
        Ok(Info {
            all: cli.check(Arg::flag("all").switch('a'))?,
            versions: cli.check(Arg::flag("versions").switch('v'))?,
            units: cli.check(Arg::flag("units").switch('u'))?,
            spec: cli.get(Arg::positional("project"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // before we gather the catalog, request a shared "READ" action to the cache
        let (_cache_rd_path, cache_rd_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::PackageCache,
            Some(PRJ_CACHE_SH_LOCK_NAME),
            true,
        )?;

        // collect all manifests available (load catalog)
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;

        let dev_prj: Option<Result<Project, Fault>> = {
            match Context::find_project_path(&current_dir().unwrap()) {
                Some(dir) => Some(Project::load(dir, true, false)),
                None => None,
            }
        };

        let mut is_local_ip = false;

        // try to auto-determine the ip (check if in a working ip)
        let prj: &Project = if let Some(spec) = &self.spec {
            // find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.translate_name(&spec.to_pkg_name())? {
                // return the highest available version
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot
                } else {
                    // try to find from downloads
                    if let Some(slot) = lvl.get_download(spec.get_version()) {
                        slot
                    } else {
                        if let Some(slot) = lvl.get_available(spec.get_version()) {
                            slot
                        } else {
                            return Err(Error::IpNotFoundInCache(spec.to_string()))?;
                        }
                    }
                }
            } else {
                return Err(Error::IpNotFoundAnywhere(
                    spec.to_string(),
                    Hint::CatalogList,
                ))?;
            }
        } else {
            if dev_prj.is_none() == true {
                return Err(Error::NoAssumedWorkingIpFound)?;
            } else {
                match &dev_prj {
                    Some(Ok(r)) => {
                        is_local_ip = true;
                        r
                    }
                    Some(Err(e)) => return Err(AnyError(format!("{}", e.to_string())))?,
                    _ => panic!("unreachable code"),
                }
            }
        };

        // load the ip's manifest
        if self.units == true {
            if prj.get_mapping().is_physical() == true {
                // try to read from cache file
                let cache_data = match is_local_ip {
                    true => None,
                    false => Project::read_cache_metadata(prj.get_root()),
                };
                if let Some(mut cache) = cache_data {
                    let mut units = cache.get_units_mut();
                    print!("{}", Self::format_cached_units_table(&mut units, self.all));
                } else {
                    // force computing the primary design units if a physical ip (non-archived)
                    let units = prj.collect_units(true, false, c.are_units_private_by_default())?;
                    print!(
                        "{}",
                        Self::format_units_table(
                            units.into_iter().map(|(_, unit)| unit).collect(),
                            self.all,
                            is_local_ip,
                        )
                    );
                }
            } else {
                // a 'virtual' ip, so try to extract units from
                crate::info!(
                    "unable to display HDL units from a downloaded project; try again after installing"
                );
            }

            // release our "READ" action to the cache
            crate::util::filesystem::release_lock(&cache_rd_lock)?;

            return Ok(());
        }

        // display all installed versions in the cache
        if self.versions == true {
            let specified_ver = if let Some(spec) = self.spec.as_ref() {
                spec.get_version().as_specific()
            } else {
                None
            };

            return match catalog.get_possible_versions(prj.get_uuid()) {
                Some(vers) => {
                    match vers.len() {
                        0 => {
                            crate::info!("no versions in the cache")
                        }
                        _ => {
                            let mut data = String::new();
                            // further restrict versions if a particular version is set
                            vers.iter()
                                .filter(move |p| {
                                    specified_ver.is_none()
                                        || version::is_compatible(
                                            specified_ver.unwrap(),
                                            &p.get_version(),
                                        ) == true
                                })
                                .for_each(|v| {
                                    data.push_str(&format!(
                                        "{:<16}{:<9}\n",
                                        v.get_version().to_string(),
                                        v.get_state().to_string()
                                    ));
                                });
                            // pop the last \n
                            data.pop();
                            println!("{}", data);
                        }
                    }
                    Ok(())
                }
                None => Err(AnyError(format!("no project found in catalog")))?,
            };
        }

        // print the manifest data "pretty"
        let s = self.format_manifest_info(&prj.get_man());
        print!("{}", s);
        Ok(())
    }
}

impl Info {
    fn format_manifest_info(&self, man: &Manifest) -> String {
        let mut result = String::new();
        let prj = man.get_project();
        result.push_str(&format!("{}", prj.get_name().to_string().green().bold()));
        // check if there are any keywords
        for key in prj.get_keywords() {
            result.push_str(&format!(" {}{}", "#".cyan().bold(), key.cyan().bold()));
        }
        result.push_str("\n");
        if let Some(desc) = prj.get_description() {
            result.push_str(desc);
            result.push_str("\n");
        }
        result.push_str(&format!(
            "{}: {}\n",
            "version".green().bold(),
            prj.get_version()
        ));
        result.push_str(&format!("{}: {}\n", "uuid".green().bold(), prj.get_uuid()));
        if let Some(lic) = prj.get_license() {
            result.push_str(&format!("{}: {}\n", "license".green().bold(), lic));
        } else {
            result.push_str(&format!(
                "{}: {}\n",
                "license".green().bold(),
                "unknown".yellow().bold()
            ));
        }
        if let Some(docs) = prj.get_documentation() {
            result.push_str(&format!("{}: {}\n", "documentation".green().bold(), docs));
        }
        // display dependencies if exist
        let deps = man.get_deps_list(false, true);
        if deps.len() > 0 {
            result.push_str(&format!("{}:", "dependencies".green().bold()));
            for (name, dep) in deps {
                result.push_str(&format!("\n    {}:{}", name, dep.get_version()));
            }
            result.push_str("\n");
        }
        result
    }

    /// Creates a string to display the primary design units for the particular ip from the cached data file.
    fn format_cached_units_table(table: &mut Vec<UnitCache>, all: bool) -> String {
        let mut result = String::new();
        table.sort_by(|a, b| match a.get_visibility().cmp(&b.get_visibility()) {
            Ordering::Equal => a.get_name().cmp(&b.get_name()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        });

        for unit in table {
            // skip this unit if it is not listed public and all is not provided
            if all == false && unit.get_visibility() != Visibility::Public {
                continue;
            }
            // let i = 15 + 6 - unit.get_symbol().len();
            // result.push_str(&format!(
            //     "{:<40}{}{:<i$}{:<9}\n",
            //     unit.get_name().to_string().green().bold(),
            //     unit.get_symbol().blue().bold(),
            //     &format!(" ({})", unit.get_lang().into_fileset_name()),
            //     unit.get_visibility().to_string(),
            // ));
            result.push_str(&format!(
                "{:<40}{:<15}{:<7}{:<9}\n",
                unit.get_name().to_string(), // .green().bold(),
                unit.get_symbol().cyan().bold(),
                unit.get_lang().into_fileset_name().bold(),
                unit.get_visibility().to_string(),
            ));
        }
        result
    }

    /// Creates a string for to display the primary design units for the particular ip.
    fn format_units_table(table: Vec<LangUnit>, all: bool, is_local_ip: bool) -> String {
        let mut result = String::new();
        let mut table = table;

        table.sort_by(|a, b| match a.get_visibility().cmp(&b.get_visibility()) {
            Ordering::Equal => a.get_name().cmp(&b.get_name()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        });

        for unit in table {
            // skip this unit if it is not listed public and all is not provided
            if is_local_ip == false && all == false && unit.get_visibility() != &Visibility::Public
            {
                continue;
            }

            // IDEA: apply no color if private or protected?
            // let yes_color = unit.get_visibility() == &Visibility::Public;
            result.push_str(&format!(
                "{:<40}{:<15}{:<7}{:<9}\n",
                unit.get_name().to_string(), // .green().bold(),
                unit.to_string().cyan().bold(),
                unit.get_lang().into_fileset_name().bold(),
                unit.get_visibility().to_string(),
            ));
        }
        result
    }
}
