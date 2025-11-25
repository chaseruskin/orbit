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

use crate::commands::helps::get;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::lang;
use crate::core::lang::sv::format::SystemVerilogFormat;
use crate::core::lang::verilog::symbols::module::Module;
use crate::core::lang::vhdl::format::VhdlFormat;
use crate::core::lang::vhdl::interface;
use crate::core::lang::vhdl::interface::Architectures;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::core::lang::vhdl::token::Identifier as VhdlIdentifier;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::core::lang::LangUnit;
use crate::core::legend::EntityJson;
use crate::core::legend::ModuleJson;
use crate::core::project::PartialProjectIdSpec;
use crate::core::project::Project;
use crate::core::project::ProjectIdSpec;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::{AnyError, Fault};
use crate::util::filesystem::LockZone;
use crate::util::filesystem::Standardize;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

/// The availale options to specify what language to return the code snippets
/// for the `get` command.
#[derive(Debug, PartialEq)]
enum LangConversion {
    Vhdl,
    Sv,
    Native,
}

impl Default for LangConversion {
    fn default() -> Self {
        Self::Native
    }
}

impl FromStr for LangConversion {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vhdl" => Ok(Self::Vhdl),
            "sv" => Ok(Self::Sv),
            "native" => Ok(Self::Native),
            _ => Err(AnyError(format!("value must be 'native', 'vhdl', or 'sv'"))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Get {
    unit: VhdlIdentifier,
    project: Option<PartialProjectIdSpec>,
    signals: bool,
    component: bool,
    instance: bool,
    library: bool,
    language: LangConversion,
    architectures: bool,
    json: bool,
    signal_prefix: String,
    signal_suffix: String,
    // NOTE: not done yet... requires more work with detecting generics in the datatype of the signals
    // const_prefix: String,
    // const_suffix: String,
    name: Option<VhdlIdentifier>,
}

impl Subcommand<Context> for Get {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(get::HELP))?;
        Ok(Self {
            signals: cli.check(Arg::flag("signals").switch('s'))?,
            component: cli.check(Arg::flag("component").switch('c'))?,
            instance: cli.check(Arg::flag("instance").switch('i'))?,
            library: cli.check(Arg::flag("library").switch('l'))?,
            architectures: cli.check(Arg::flag("architecture").switch('a'))?,
            json: cli.check(Arg::flag("json"))?,
            language: cli
                .get(Arg::option("language").value("hdl"))?
                .unwrap_or_default(),
            signal_prefix: cli
                .get(Arg::option("signal-prefix").value("str"))?
                .unwrap_or_default(),
            signal_suffix: cli
                .get(Arg::option("signal-suffix").value("str"))?
                .unwrap_or_default(),
            // const_prefix: cli
            //     .get(Arg::option("const-prefix").value("value"))?
            //     .unwrap_or_default(),
            // const_suffix: cli
            //     .get(Arg::option("const-suffix").value("value"))?
            //     .unwrap_or_default(),
            project: cli.get(Arg::option("project").switch('p').value("spec"))?,
            name: cli.get(Arg::option("name").value("identifier"))?,
            unit: cli.require(Arg::positional("unit"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // --name can only be used with --instance is set
        if self.name.is_some() && self.instance == false {
            return Err(AnyError(format!(
                "'{}' can only be used with '{}'",
                "--name".yellow(),
                "--instance".yellow()
            )))?;
        }

        // before we gather the catalog, request an exclusive "APPEND" action to the cache
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // gather the catalog
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;

        let mut is_local_ip = false;
        // try to auto-determine the ip (check if in a working ip)
        let ip_path = if let Some(spec) = &self.project {
            // find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.translate_name(&spec.to_pkg_name())? {
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot.get_root().clone()
                } else if let Some(_) = lvl.get_download(spec.get_version()) {
                    return Err(AnyError(format!(
                        "project {} is downloaded but not installed: install the project to use this command",
                        spec
                    )))?;
                } else if let Some(_) = lvl.get_available(spec.get_version()) {
                    return Err(AnyError(format!(
                        "project {} is available but not installed: install the project to use this command",
                        spec
                    )))?;
                } else {
                    return Err(AnyError(format!(
                        "project {} does not exist in the cache",
                        spec
                    )))?;
                }
            } else {
                return Err(AnyError(format!("no project found in cache")))?;
            }
        } else {
            let ip = Context::find_project_path(&env::current_dir().unwrap());
            is_local_ip = true;
            if ip.is_none() == true {
                return Err(AnyError(format!("no project provided or detected")))?;
            } else {
                ip.unwrap()
            }
        };

        // load the manifest from the path
        let ip = Project::load(ip_path, is_local_ip, false)?;

        let result = self.run(&ip, is_local_ip, &c);

        // release our "APPEND" action to the cache
        crate::util::filesystem::release_lock(&cache_ap_lock)?;

        result
    }
}

impl Get {
    fn run(&self, project: &Project, is_local: bool, c: &Context) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let selected_unit = Self::fetch_entity(
            &project,
            &LangIdentifier::Vhdl(self.unit.clone()),
            c.are_units_private_by_default(),
        )?;
        let unit = match selected_unit {
            Some(lu) => {
                // verify the unit is only set to public visibility when outside of ip
                if is_local == false && lu.get_visibility().is_public() == false {
                    return Err(Error::UnitIsWrongVisibility(
                        String::from("get"),
                        lu.get_name(),
                        lu.get_visibility().clone(),
                        Hint::ShowAvailableUnitsExternal(
                            project.get_man().get_project().into_project_id_spec(),
                        ),
                    ))?;
                }
                // check to make sure it is a component
                if lu.is_component() {
                    lu
                } else {
                    let hint = match is_local {
                        true => Hint::ShowAvailableUnitsLocal,
                        false => Hint::ShowAvailableUnitsExternal(
                            project.get_man().get_project().into_project_id_spec(),
                        ),
                    };
                    return Err(Error::GetUnitNotComponent(lu.get_name().to_string(), hint))?;
                }
            }
            None => {
                let hint = match is_local {
                    true => Hint::ShowAvailableUnitsLocal,
                    false => Hint::ShowAvailableUnitsExternal(
                        project.get_man().get_project().into_project_id_spec(),
                    ),
                };
                return Err(Error::GetUnitNotFound(self.unit.to_string(), hint))?;
            }
        };

        let source_file = unit.get_source_file();

        // determine how to handle unit display
        match unit.get_lang() {
            Lang::Vhdl => {
                let entity = unit.get_vhdl_symbol().unwrap().as_entity().unwrap();

                match self.language {
                    LangConversion::Sv => {
                        // convert the entity to a SV module
                        let module = entity.to_sv_module()?;
                        self.display_verilog_module(
                            &project,
                            &module,
                            &c.get_sv_format(),
                            source_file,
                        )
                    }
                    _ => self.display_vhdl_entity(
                        &project,
                        entity,
                        is_local,
                        &c.get_vhdl_format(),
                        source_file,
                    ),
                }
            }
            Lang::Verilog => {
                let module = unit.get_verilog_symbol().unwrap().as_module().unwrap();

                match self.language {
                    LangConversion::Vhdl => {
                        // convert the entity to a SV module
                        let entity = module.to_vhdl_entity()?;
                        self.display_vhdl_entity(
                            &project,
                            &entity,
                            is_local,
                            &c.get_vhdl_format(),
                            source_file,
                        )
                    }
                    _ => self.display_verilog_module(
                        &project,
                        module,
                        &c.get_sv_format(),
                        source_file,
                    ),
                }
            }
            Lang::SystemVerilog => {
                let module = unit
                    .get_systemverilog_symbol()
                    .unwrap()
                    .as_module()
                    .unwrap();

                match self.language {
                    LangConversion::Vhdl => {
                        // convert the entity to a SV module
                        let entity = module.to_vhdl_entity()?;
                        self.display_vhdl_entity(
                            &project,
                            &entity,
                            is_local,
                            &c.get_vhdl_format(),
                            source_file,
                        )
                    }
                    _ => self.display_verilog_module(
                        &project,
                        module,
                        &c.get_sv_format(),
                        source_file,
                    ),
                }
            }
        }?;

        Ok(())
    }

    fn display_vhdl_entity(
        &self,
        ip: &Project,
        entity: &Entity,
        is_local: bool,
        fmt: &VhdlFormat,
        source: &str,
    ) -> Result<(), Fault> {
        // determine if default print should appear
        let default_output = self.architectures == false
            && self.instance == false
            && self.json == false
            && self.signals == false
            && self.component == false
            && self.library == false;

        // make the library reference the current worki ng ip 'work' if its internal
        let lib = match is_local {
            true => VhdlIdentifier::new_working(),
            false => ip
                .get_man()
                .get_hdl_library()
                .as_vhdl_name()
                .unwrap()
                .clone(),
        };

        // Track if we need to add a "\n" before the next series of text outputs
        let mut need_sep: bool = false;

        // display architectures
        if self.architectures == true {
            print!("{}", entity.get_architectures());
            need_sep = true;
        }

        if fmt.is_syntax_highlighted() == false {
            // force turn off coloring output
            colored::control::set_override(false);
        }

        // display library declaration line if displaying instance
        if self.library == true {
            if need_sep == true {
                println!();
            }
            print!("{}", interface::library_statement(&lib));
            need_sep = true;
        }

        // display component declaration
        if self.component == true || default_output == true {
            if need_sep == true {
                println!();
            }
            print!("{}", entity.into_component(&fmt));
            need_sep = true;
        }

        // display signal declarations
        if self.signals == true {
            let constants = entity.into_constants(&fmt, "", "");
            if constants.is_empty() == false {
                if need_sep == true {
                    println!();
                }
                print!("{}", constants);
                need_sep = true;
            }
            let signals = entity.into_signals(&fmt, &self.signal_prefix, &self.signal_suffix);
            if signals.is_empty() == false {
                if need_sep == true {
                    println!();
                }
                print!("{}", signals);
                need_sep = true;
            }
        }

        // only display the direct entity instantiation code if not providing component code
        let lib = if self.component == true {
            None
        } else {
            Some(lib)
        };

        // display instantiation code
        if self.instance == true {
            if need_sep == true {
                println!();
            }
            println!(
                "{}",
                entity.into_instance(
                    &self.name,
                    &lib,
                    &fmt,
                    &self.signal_prefix,
                    &self.signal_suffix,
                    "",
                    "",
                )
            );
            need_sep = true;
        }

        // print as json data
        if self.json == true {
            if need_sep == true {
                println!();
            }
            println!(
                "{}",
                serde_json::to_string(&EntityJson::new(&entity, &source))?
            );
        }

        Ok(())
    }

    fn display_verilog_module(
        &self,
        _ip: &Project,
        module: &Module,
        fmt: &SystemVerilogFormat,
        source: &str,
    ) -> Result<(), Fault> {
        // determine if default print should appear
        let default_output = self.architectures == false
            && self.instance == false
            && self.json == false
            && self.signals == false
            && self.component == false
            && self.library == false;

        // Track if we need to add a "\n" before the next series of text outputs
        let mut need_sep: bool = false;

        // display architectures
        if self.architectures == true {
            print!("{}N/A\n", Architectures::new(&Vec::new()));
            need_sep = true;
        }

        if self.component == true || default_output == true {
            if need_sep == true {
                println!();
            }
            print!("{}\n", module.into_declaration(&fmt));
            need_sep = true;
        }

        if self.signals == true {
            if need_sep == true {
                println!();
            }
            print!(
                "{}",
                module.into_wires(&self.signal_prefix, &self.signal_suffix, &fmt)
            );
            need_sep = true;
        }

        if self.instance == true {
            if need_sep == true {
                println!();
            }
            println!(
                "{}",
                module.into_instance(&self.name, &self.signal_prefix, &self.signal_suffix, &fmt)
            );
            need_sep = true;
        }

        // print as json data
        if self.json == true {
            if need_sep == true {
                println!();
            }
            println!(
                "{}",
                serde_json::to_string(&ModuleJson::new(&module, &source))?
            );
        }

        Ok(())
    }

    fn fetch_entity(
        ip: &Project,
        name: &LangIdentifier,
        priv_by_default: bool,
    ) -> Result<Option<LangUnit>, Fault> {
        // check if we can use the cached metadata
        if let Some(cached) = Project::read_cache_metadata(ip.get_root()) {
            let units = cached.get_units();
            if let Some(unit) = units.iter().find(|p| &p.get_name() == name) {
                let files = unit
                    .get_sources()
                    .iter()
                    .map(|f| {
                        if PathBuf::from(f).is_relative() {
                            PathBuf::standardize(ip.get_root().join(f))
                                .as_os_str()
                                .to_string_lossy()
                                .to_string()
                        } else {
                            f.to_string()
                        }
                    })
                    .collect();
                let mut mapping = lang::collect_units(&files)?;
                let result = mapping.remove(name);
                Ok(result)
            } else {
                Ok(None)
            }
        } else {
            let mut mapping = ip.collect_units(true, false, priv_by_default)?;
            let result = mapping.remove(name);
            Ok(result)
        }
    }
}

#[derive(Debug)]
pub enum GetError {
    UnitNotFound(LangIdentifier, ProjectIdSpec),
    SuggestShow(String, Hint),
}

impl std::error::Error for GetError {}

impl std::fmt::Display for GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnitNotFound(ent, spec) => {
                write!(f, "failed to find unit \"{}\" in project \"{}\"", ent, spec)
            }
            Self::SuggestShow(err, hint) => {
                write!(f, "{}{}", err, hint)
            }
        }
    }
}
