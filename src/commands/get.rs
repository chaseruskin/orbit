use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::helps::get;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::parser::Symbol;
use crate::core::lang::vhdl::format::VhdlFormat;
use crate::core::lang::vhdl::interface;
use crate::core::lang::vhdl::primaryunit::VhdlIdentifierError;
use crate::core::lang::vhdl::symbols::architecture::Architecture;
use crate::core::lang::vhdl::symbols::entity::Entity;
use crate::core::lang::vhdl::symbols::VHDLParser;
use crate::core::lang::vhdl::symbols::VhdlSymbol;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::manifest::FromFile;
use crate::core::manifest::Manifest;
use crate::core::manifest::IP_MANIFEST_FILE;
use crate::core::pkgid::PkgPart;
use crate::core::version::Version;
use crate::util::anyerror::{AnyError, Fault};
use crate::OrbitResult;
use clif::arg::{Flag, Optional, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use colored::Colorize;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Get {
    unit: Identifier,
    ip: Option<PartialIpSpec>,
    signals: bool,
    component: bool,
    instance: bool,
    architectures: bool,
    json: bool,
    // info: bool,
    name: Option<Identifier>,
}

impl FromCli for Get {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(get::HELP).ref_usage(2..4))?;
        let command = Ok(Self {
            signals: cli.check_flag(Flag::new("signals").switch('s'))?,
            component: cli.check_flag(Flag::new("component").switch('c'))?,
            instance: cli.check_flag(Flag::new("instance").switch('i'))?,
            architectures: cli.check_flag(Flag::new("architecture").switch('a'))?,
            json: cli.check_flag(Flag::new("json"))?,
            // info: cli.check_flag(Flag::new("info"))?, // @todo: implement
            ip: cli.check_option(Optional::new("ip").value("spec"))?,
            name: cli.check_option(Optional::new("name").value("identifier"))?,
            unit: cli.require_positional(Positional::new("unit"))?,
        });
        command
    }
}

use crate::core::lang::parser::Parse;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use std::env;

impl Command<Context> for Get {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
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

        // try to auto-determine the ip (check if in a working ip)
        let ip_path = if let Some(spec) = &self.ip {
            // @todo: find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.inner().get(spec.get_name()) {
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot.get_root().clone()
                } else {
                    return Err(AnyError(format!("IP {} does not exist in the cache", spec)))?;
                }
            } else {
                return Err(AnyError(format!("no ip found in cache")))?;
            }
        } else {
            let ip = Context::find_ip_path(&env::current_dir().unwrap());
            if ip.is_none() == true {
                return Err(AnyError(format!("no ip provided or detected")))?;
            } else {
                ip.unwrap()
            }
        };

        // load the manifest from the path
        let man = Manifest::from_file(&ip_path.join(IP_MANIFEST_FILE))?;

        let default_fmt = VhdlFormat::new();
        let fmt = match c.get_config().get_vhdl_formatting() {
            Some(v) => v,
            None => &default_fmt,
        };
        self.run(man, &ip_path, &fmt)
    }
}

impl Get {
    fn run(&self, man: Manifest, dir: &PathBuf, fmt: &VhdlFormat) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let ent = match Self::fetch_entity(&self.unit, &dir, &man) {
            Ok(r) => r,
            Err(e) => {
                return Err(GetError::SuggestShow(
                    e.to_string(),
                    man.get_ip().get_name().clone(),
                    man.get_ip().get_version().clone(),
                ))?
            }
        };

        // add to dependency list if within a ip and `self.add` is `true`
        // if let Some(mut cur_ip) = current_ip {
        //     // verify it is the not the same package! and we explicitly want to add
        //     if cur_ip.get_pkgid() != ip.get_pkgid() && self.add == true {
        //         cur_ip.insert_dependency(ip.get_pkgid().clone(), self.version.as_ref().unwrap_or(&AnyVersion::Latest).clone());
        //         cur_ip.get_manifest_mut().save()?;
        //     }
        // }

        // make the library reference the current working ip 'work' if its internal
        let lib = match self.ip.is_none() {
            true => Identifier::new_working(),
            false => match man.get_ip().get_library() {
                Some(lib) => Identifier::from(lib),
                // default to the package's name
                None => Identifier::from(man.get_ip().get_name()),
            },
        };

        // display architectures
        if self.architectures == true {
            println!("{}", ent.get_architectures());
        }

        if fmt.is_syntax_highlighted() == false {
            // force turn off coloring output
            colored::control::set_override(false);
        }

        // display component declaration
        if self.component == true {
            println!("{}", ent.into_component(&fmt));
        // display library declaration line if displaying instance
        } else if self.instance == true {
            println!("{}", interface::library_statement(&lib));
        }

        // display signal declarations
        if self.signals == true {
            let constants = ent.into_constants(&fmt);
            if constants.is_empty() == false {
                println!("{}", constants);
            }
            let signals = ent.into_signals(&fmt);
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
            println!("{}", ent.into_instance(&self.name, lib, &fmt));
        }

        // print as json data
        if self.json == true {
            println!("{}", serde_json::to_string_pretty(&ent)?);
        }

        Ok(())
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &Identifier, dir: &PathBuf, man: &Manifest) -> Result<Entity, Fault> {
        let files = crate::util::filesystem::gather_current_files(&dir, false);
        // @todo: generate all units first (store architectures, and entities, and then process)
        let mut result: Option<(String, Entity)> = None;
        // store map of all architectures while parsing all code
        let mut architectures: HashMap<Identifier, Vec<Architecture>> = HashMap::new();
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
                    "{}\n\nTry `orbit show {} --units` to see a list of primary design units",
                    err, spec
                )
            }
        }
    }
}

//  --add                   add dependency to Orbit.toml table

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn serialize_entity() {
        const EXPECTED_STR: &str = r#"{
  "entity": "or_gate",
  "generics": [
    {
      "name": "N",
      "mode": "in",
      "type": "positive",
      "default": "8"
    }
  ],
  "ports": [
    {
      "name": "a",
      "mode": "in",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    },
    {
      "name": "b",
      "mode": "in",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    },
    {
      "name": "q",
      "mode": "out",
      "type": "std_logic_vector(N-1 downto 0)",
      "default": null
    }
  ],
  "architectures": [
    "rtl",
    "other"
  ]
}"#;
        let ent = Get::fetch_entity(
            &Identifier::from_str("or_gate").unwrap(),
            &PathBuf::from("./tests/data/gates"),
            &Manifest::new(),
        )
        .unwrap();
        let json_str = serde_json::to_string_pretty(&ent).unwrap();
        assert_eq!(json_str, EXPECTED_STR);
    }
}
