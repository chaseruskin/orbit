use std::collections::HashMap;
use std::path::PathBuf;

use colored::Colorize;
use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::catalog::CatalogError;
use crate::core::manifest::IpManifest;
use crate::core::parser::Symbol;
use crate::core::version::AnyVersion;
use crate::core::version::Version;
use crate::core::lang::vhdl::interface;
use crate::core::lang::vhdl::primaryunit::VhdlIdentifierError;
use crate::core::lang::vhdl::symbol::Architecture;
use crate::core::lang::vhdl::symbol::Entity;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::{AnyError, Fault};

#[derive(Debug, PartialEq)]
pub struct Get {
    unit: Identifier,
    ip: Option<PkgId>,
    signals: bool,
    component: bool,
    instance: bool,
    architectures: bool,
    version: Option<AnyVersion>,
    info: bool,
    add: bool,
    name: Option<Identifier>,
}

impl FromCli for Get {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Get {
            signals: cli.check_flag(Flag::new("signals").switch('s'))?,
            component: cli.check_flag(Flag::new("component").switch('c'))?,
            instance: cli.check_flag(Flag::new("instance").switch('i'))?,
            architectures: cli.check_flag(Flag::new("architecture").switch('a'))?,
            version: cli.check_option(Optional::new("variant").switch('v').value("version"))?,
            info: cli.check_flag(Flag::new("info"))?, // @todo: implement
            ip: cli.check_option(Optional::new("ip").value("pkgid"))?,
            add: cli.check_flag(Flag::new("add"))?,
            name: cli.check_option(Optional::new("name").value("identifier"))?,
            unit: cli.require_positional(Positional::new("unit"))?,
        });
        command
    }
}

use crate::core::parser::Parse;
use crate::core::lang::vhdl;
use crate::core::lang::vhdl::symbol;
use crate::core::lang::vhdl::token::VHDLTokenizer;

impl Command for Get {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {  
        // --name can only be used with --instance is set
        if self.name.is_some() && self.instance == false {
            return Err(AnyError(format!("'{}' can only be used with '{}'", "--name".yellow(), "--instance".yellow())))?
        }

        let current_ip = match c.goto_ip_path() {
            Ok(_) => Some(IpManifest::from_path(c.get_ip_path().unwrap())?),
            Err(_) => None,
        };

        // verify --add is used within an ip
        if self.add == true && current_ip.is_none() {
            return Err(AnyError(format!("'{}' can only be used inside an ip directory", "--add".yellow())))?
        }

        // must be in an IP if omitting the pkgid
        if self.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }
            // do not add self to requirements
            self.run(&IpManifest::from_path(c.get_ip_path().unwrap())?, true, None, &AnyVersion::Dev) 
        // checking external IP dependency
        } else {
            // gather the catalog (all manifests)
            let catalog = Catalog::new()
                .store(c.get_store_path())
                .development(c.get_development_path().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?;
            let ids = catalog.inner().keys().map(|f| { f }).collect();
            let target = crate::core::ip::find_ip(&self.ip.as_ref().unwrap(), ids)?;
            
            // find all manifests and prioritize installed manifests over others but to help with errors/confusion
            let status = catalog.inner().get(&target).unwrap();

            // determine version to grab
            let v = self.version.as_ref().unwrap_or(&AnyVersion::Latest);
            let ip = match v {
                AnyVersion::Dev => status.get_dev(),
                _ => status.get_install(v)
            };
            if ip.is_none() {
                // check if the ip is available for more helpful error message
                return match status.is_available_or_in_store(catalog.get_store(), &target, v) {
                    true => Err(CatalogError::SuggestInstall(target.to_owned(), v.to_owned())),
                    false => Err(CatalogError::NoVersionForIp(target.to_owned(), v.to_owned()))
                }?
            }

            self.run(ip.unwrap(), false, current_ip, v)
        }
    }
}

impl Get {
    fn run(&self, ip: &IpManifest, is_self: bool, current_ip: Option<IpManifest>, ver: &AnyVersion) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let ent = match Self::fetch_entity(&self.unit, &ip) {
            Ok(r) => r,
            Err(e) => return Err(GetError::SuggestProbe(e.to_string(), ip.get_pkgid().clone(), ver.clone()))?
        };

        // add to dependency list if within a ip and `self.add` is `true`
        if let Some(mut cur_ip) = current_ip {
            // verify it is the not the same package! and we explicitly want to add 
            if cur_ip.get_pkgid() != ip.get_pkgid() && self.add == true {
                cur_ip.insert_dependency(ip.get_pkgid().clone(), self.version.as_ref().unwrap_or(&AnyVersion::Latest).clone());
                cur_ip.get_manifest_mut().save()?;
            }
        }

        // make the library reference the current working ip 'work' if its internal
        let lib = match is_self {
            true => Identifier::new_working(),
            false => Identifier::from(ip.get_pkgid().get_library().as_ref().unwrap()),
        };
        
        // display architectures    
        if self.architectures == true {
            println!("{}", ent.get_architectures());
        }

        // display component declaration
        if self.component == true {
            println!("{}", ent.into_component());
        // display library declaration line if displaying instance
        } else if self.instance == true {
            println!("{}", interface::library_statement(&lib));
        }

        // display signal declarations
        if self.signals == true {
            let constants = ent.into_constants();
            if constants.is_empty() == false {
                println!("{}", constants);
            }
            let signals = ent.into_signals();
            if signals.is_empty() == false {
                println!("{}", signals);
            }
        }  

        // only display the direct entity instantiation code if not providing component code
        let lib = if self.component == true { None } else { Some(lib) };

        // display instantiation code
        if self.instance == true {
            let name = match &self.name {
                Some(iden) => iden.clone(),
                None => Identifier::Basic("uX".to_string()),
            };
            println!("{}", ent.into_instance(&name, lib));
        }

        Ok(())
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &Identifier, ip: &IpManifest) -> Result<symbol::Entity, Fault> {
        let files = crate::util::filesystem::gather_current_files(&ip.get_root());
        // @todo: generate all units first (store architectures, and entities, and then process)
        let mut result: Option<(String, Entity)> = None;
        // store map of all architectures while parsing all code
        let mut architectures: HashMap<Identifier, Vec<Architecture>> = HashMap::new();
        for f in files {
            // lex and parse
            if crate::core::fileset::is_vhdl(&f) == true {
                let text = std::fs::read_to_string(&f)?;
            
                // pull all architectures
                let units: Vec<Symbol<symbol::VHDLSymbol>> = vhdl::symbol::VHDLParser::parse(VHDLTokenizer::from_source_code(&text).into_tokens())
                    .into_iter()
                    .filter_map(|f| if f.is_ok() { 
                        let unit = f.unwrap();
                        match unit.as_ref().as_architecture() {
                            Some(_) => {
                                let arch = unit.take().into_architecture().unwrap();
                                match architectures.get_mut(arch.entity()) {
                                    Some(list) => { list.push(arch); () },
                                    None => { architectures.insert(arch.entity().clone(), vec![arch]); () }
                                }
                                None 
                            },
                            None => Some(unit)
                        }
                    } else { None }
                    ).collect();

                // detect entity
                let requested_entity = units
                    .into_iter() 
                    .filter_map(|r| r.take().into_entity())
                    .find(|p| p.get_name() == iden);

                // verify entity was not already detected (duplicate)
                if let Some(ent) = requested_entity {
                    match result {
                        Some((src_file, dupe)) => return Err(VhdlIdentifierError::DuplicateIdentifier(dupe.get_name().clone(), PathBuf::from(src_file), dupe.get_position().clone(), PathBuf::from(f), ent.get_position().clone()))?,
                        None => result = Some((f, ent)),
                    }
                }
            }
        }
        match result {
            Some((_, mut entity)) => {
                match architectures.remove(entity.get_name()) {
                    Some(archs) => for arch in archs { entity.link_architecture(arch) }
                    None => (),
                }
                Ok(entity)
            }
            None => Err(GetError::EntityNotFound(iden.clone(), ip.get_pkgid().clone(), ip.get_version().clone()))?
        }
    }
}

#[derive(Debug)]
pub enum GetError {
    EntityNotFound(Identifier, PkgId, Version),
    SuggestProbe(String, PkgId, AnyVersion),
}

impl std::error::Error for GetError {}

impl std::fmt::Display for GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityNotFound(ent, pkg, ver) => {
                write!(f, "entity '{0}' is not found in ip '{1}' under version '{2}'", ent, pkg, ver)
            },
            Self::SuggestProbe(err, pkg, ver) => {
                write!(f, "{}\n\nTry `orbit probe {1} -v {2} --units` to see a list of primary design units", err, pkg, ver)
            },
        }
    }
}

const HELP: &str = "\
Fetch an hdl entity for code integration.

Usage:
    orbit get [options] <unit>

Args:
    <unit>                  entity identifier

Options:
    --ip <pkgid>            ip to reference unit from
    --variant, -v <version> ip version to use
    --component, -c         print component declaration
    --signals,   -s         print signal declarations
    --instance,  -i         print instantation
    --info                  access code file's header comment
    --architecture, -a      print available architectures
    --add                   add the ip to the Orbit.toml dependency table
    --name <identifier>     specific instance identifier

Use 'orbit help get' to learn more about the command.
";


// #[cfg(test)]
// mod test {
//     // use super::*;

//     // use std::str::FromStr;

//     // #[test]
//     // #[ignore]
//     // fn fetch_entity() {
//     //     let _ = Get::fetch_entity(&Identifier::from_str("or_gate").unwrap(), &std::path::PathBuf::from("./test/data/gates")).unwrap();
//     //     panic!("inspect")
//     // }
// }