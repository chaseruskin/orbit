use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::helps::get;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::parser::Parse;
use crate::core::lang::parser::Symbol;
use crate::core::lang::verilog::symbols::module::Module;
use crate::core::lang::verilog::token::identifier::Identifier as VerilogIdentifier;
use crate::core::lang::vhdl::format::VhdlFormat;
use crate::core::lang::vhdl::interface;
use crate::core::lang::vhdl::primaryunit::VhdlIdentifierError;
use crate::core::lang::vhdl::symbols::architecture::Architecture;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::core::lang::vhdl::symbols::VHDLParser;
use crate::core::lang::vhdl::symbols::VhdlSymbol;
use crate::core::lang::vhdl::token::Identifier as VhdlIdentifier;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use crate::core::lang::Lang;
use crate::core::lang::LangUnit;
use crate::core::lang::Language;
use crate::core::manifest::Manifest;
use crate::core::pkgid::PkgPart;
use crate::core::version::Version;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::{AnyError, Fault};
use colored::Colorize;
use std::env;
use std::str::FromStr;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Get {
    unit: VhdlIdentifier,
    ip: Option<PartialIpSpec>,
    signals: bool,
    component: bool,
    instance: bool,
    library: bool,
    architectures: bool,
    json: bool,
    signal_prefix: String,
    signal_suffix: String,
    // @note: not done yet... requires more work with detecting generics in the datatype of the signals
    // const_prefix: String,
    // const_suffix: String,
    // info: bool,
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
            // info: cli.check(Arg::flag("info"))?, // @todo: implement
            signal_prefix: cli
                .get(Arg::option("signal-prefix").value("value"))?
                .unwrap_or_default(),
            signal_suffix: cli
                .get(Arg::option("signal-suffix").value("value"))?
                .unwrap_or_default(),
            // const_prefix: cli
            //     .get(Arg::option("const-prefix").value("value"))?
            //     .unwrap_or_default(),
            // const_suffix: cli
            //     .get(Arg::option("const-suffix").value("value"))?
            //     .unwrap_or_default(),
            ip: cli.get(Arg::option("ip").value("spec"))?,
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

        // @todo: load the catalog
        let catalog = Catalog::new()
            // .store(c.get_store_path())
            // .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?;

        let mut is_local_ip = false;
        // try to auto-determine the ip (check if in a working ip)
        let ip_path = if let Some(spec) = &self.ip {
            // @todo: find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.inner().get(spec.get_name()) {
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot.get_root().clone()
                } else {
                    return Err(AnyError(format!("ip {} does not exist in the cache", spec)))?;
                }
            } else {
                return Err(AnyError(format!("no ip found in cache")))?;
            }
        } else {
            let ip = Context::find_ip_path(&env::current_dir().unwrap());
            is_local_ip = true;
            if ip.is_none() == true {
                return Err(AnyError(format!("no ip provided or detected")))?;
            } else {
                ip.unwrap()
            }
        };

        // load the manifest from the path
        let ip = Ip::load(ip_path, is_local_ip)?;

        let default_fmt = VhdlFormat::new();
        let fmt = match c.get_config().get_vhdl_formatting() {
            Some(v) => v,
            None => &default_fmt,
        };
        self.run(&ip, &fmt, &c.get_languages(), is_local_ip)
    }
}

impl Get {
    fn run(
        &self,
        ip: &Ip,
        vhdl_fmt: &VhdlFormat,
        lang: &Language,
        is_local: bool,
    ) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let selected_unit = Self::fetch_entity_2(
            &ip,
            &LangIdentifier::Vhdl(self.unit.clone()),
            lang,
            is_local,
        )?;
        let unit = match selected_unit {
            Some(r) => {
                // check to make sure it is a component
                if r.is_usable_component() {
                    r
                } else {
                    let hint = match is_local {
                        true => Hint::ShowAvailableUnitsLocal,
                        false => {
                            Hint::ShowAvailableUnitsExternal(ip.get_man().get_ip().into_ip_spec())
                        }
                    };
                    return Err(Error::GetUnitNotComponent(r.get_name().to_string(), hint))?;
                }
            }
            None => {
                let hint = match is_local {
                    true => Hint::ShowAvailableUnitsLocal,
                    false => Hint::ShowAvailableUnitsExternal(ip.get_man().get_ip().into_ip_spec()),
                };
                return Err(Error::GetUnitNotFound(self.unit.to_string(), hint))?;
            }
        };

        // determine how to handle unit display
        let result = match unit.get_lang() {
            Lang::Vhdl => self.display_vhdl_entity(
                &ip,
                unit.get_vhdl_symbol().unwrap().as_entity().unwrap(),
                is_local,
                vhdl_fmt,
            ),
            Lang::Verilog => self.display_verilog_module(
                &ip,
                unit.get_verilog_symbol().unwrap().as_module().unwrap(),
            ),
            Lang::SystemVerilog => {
                todo!()
            }
        }?;

        // let ent = match Self::fetch_entity(&self.unit, &dir, &man) {
        //     Ok(r) => r,
        //     Err(e) => {
        //         return Err(GetError::SuggestShow(
        //             e.to_string(),
        //             man.get_ip().get_name().clone(),
        //             man.get_ip().get_version().clone(),
        //         ))?
        //     }
        // };

        Ok(())
    }

    fn display_vhdl_entity(
        &self,
        ip: &Ip,
        entity: &Entity,
        is_local: bool,
        fmt: &VhdlFormat,
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

        // display architectures
        if self.architectures == true {
            println!("{}", entity.get_architectures());
        }

        if fmt.is_syntax_highlighted() == false {
            // force turn off coloring output
            colored::control::set_override(false);
        }

        // display library declaration line if displaying instance
        if self.library == true {
            println!("{}", interface::library_statement(&lib));
        }

        // display component declaration
        if self.component == true || default_output == true {
            println!("{}", entity.into_component(&fmt));
        }

        // display signal declarations
        if self.signals == true {
            let constants = entity.into_constants(&fmt, "", "");
            if constants.is_empty() == false {
                println!("{}", constants);
            }
            let signals = entity.into_signals(&fmt, &self.signal_prefix, &self.signal_suffix);
            if signals.is_empty() == false {
                println!("{}", signals);
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
        }

        // print as json data
        if self.json == true {
            println!("{}", serde_json::to_string_pretty(&entity)?);
        }
        todo!()
    }

    fn display_verilog_module(&self, ip: &Ip, module: &Module) -> Result<(), Fault> {
        // determine if default print should appear
        let default_output = self.architectures == false
            && self.instance == false
            && self.json == false
            && self.signals == false
            && self.component == false
            && self.library == false;

        if self.component == true || default_output == true {
            println!("{}", module.into_declaration());
        }

        Ok(())
    }

    fn fetch_entity_2(
        ip: &Ip,
        name: &LangIdentifier,
        lang: &Language,
        is_local: bool,
    ) -> Result<Option<LangUnit>, Fault> {
        let mut files = Ip::collect_units(
            true,
            ip.get_root(),
            lang,
            is_local == false,
            ip.into_public_list(),
        )?;
        let result = files.remove(name);
        Ok(result)
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &VhdlIdentifier, dir: &PathBuf, man: &Manifest) -> Result<Entity, Fault> {
        let files = crate::util::filesystem::gather_current_files(&dir, false);
        // @todo: generate all units first (store architectures, and entities, and then process)
        let mut result: Option<(String, Entity)> = None;
        // store map of all architectures while parsing all code
        let mut architectures: HashMap<VhdlIdentifier, Vec<Architecture>> = HashMap::new();
        for f in files {
            // lex and parse VHDL files
            if crate::core::fileset::is_vhdl(&f) == true {
                let text = std::fs::read_to_string(&f)?;

                // pull all architectures
                let units: Vec<Symbol<VhdlSymbol>> =
                    VHDLParser::parse(VhdlTokenizer::from_str(&text)?.into_tokens())
                        .into_iter()
                        .filter_map(|f| {
                            if f.is_ok() {
                                let unit = f.unwrap();
                                match unit.as_ref().as_architecture() {
                                    Some(_) => {
                                        let arch = unit.take().into_architecture().unwrap();
                                        match architectures.get_mut(arch.entity()) {
                                            Some(list) => {
                                                list.push(arch);
                                                ()
                                            }
                                            None => {
                                                architectures
                                                    .insert(arch.entity().clone(), vec![arch]);
                                                ()
                                            }
                                        }
                                        None
                                    }
                                    None => Some(unit),
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                // detect entity
                let requested_entity = units
                    .into_iter()
                    .filter_map(|r| r.take().into_entity())
                    .find(|p| p.get_name() == iden);

                // verify entity was not already detected (duplicate)
                if let Some(ent) = requested_entity {
                    match result {
                        Some((src_file, dupe)) => {
                            return Err(VhdlIdentifierError::DuplicateIdentifier(
                                dupe.get_name().to_string(),
                                PathBuf::from(src_file),
                                dupe.get_position().clone(),
                                PathBuf::from(f),
                                ent.get_position().clone(),
                            ))?
                        }
                        None => result = Some((f, ent)),
                    }
                }
            // lex and parse verilog files
            } else if crate::core::fileset::is_verilog(&f) == true {
                let _text = std::fs::read_to_string(&f)?;
            }
        }
        // @MARK: do not show results if the entity is private

        match result {
            Some((_, mut entity)) => {
                match architectures.remove(entity.get_name()) {
                    Some(archs) => {
                        for arch in archs {
                            entity.link_architecture(arch)
                        }
                    }
                    None => (),
                }
                Ok(entity)
            }
            None => Err(GetError::UnitNotFound(
                iden.clone().into_lang_id(),
                man.get_ip().get_name().clone(),
                man.get_ip().get_version().clone(),
            ))?,
        }
    }
}

use crate::core::lang::LangIdentifier;

#[derive(Debug)]
pub enum GetError {
    UnitNotFound(LangIdentifier, PkgPart, Version),
    SuggestShow(String, PkgPart, Version),
}

use crate::core::ip::IpSpec;

impl std::error::Error for GetError {}

impl std::fmt::Display for GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnitNotFound(ent, pkg, ver) => {
                let spec = IpSpec::new(pkg.clone(), ver.clone());
                write!(f, "Failed to find unit '{}' in ip '{}'", ent, spec)
            }
            Self::SuggestShow(err, pkg, ver) => {
                let spec = IpSpec::new(pkg.clone(), ver.clone());
                write!(
                    f,
                    "{}\n\ntry `orbit view {} --units` to see a list of primary design units",
                    err, spec
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HdlComponent {
    Entity(Entity),
    Module(Module),
}

impl HdlComponent {
    pub fn get_name(&self) -> &LangIdentifier {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn serialize_entity() {
        const EXPECTED_STR: &str = r#"{
  "identifier": "or_gate",
  "generics": [
    {
      "identifier": "N",
      "mode": "in",
      "type": "positive",
      "default": "8"
    }
  ],
  "ports": [
    {
      "identifier": "a",
      "mode": "in",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    },
    {
      "identifier": "b",
      "mode": "in",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    },
    {
      "identifier": "q",
      "mode": "out",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    }
  ],
  "architectures": [
    "rtl",
    "other"
  ],
  "language": "vhdl"
}"#;
        let ent = Get::fetch_entity(
            &VhdlIdentifier::from_str("or_gate").unwrap(),
            &PathBuf::from("./tests/data/gates"),
            &Manifest::new(),
        )
        .unwrap();
        let json_str = serde_json::to_string_pretty(&ent).unwrap();
        assert_eq!(json_str, EXPECTED_STR);
    }
}
