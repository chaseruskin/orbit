use colored::Colorize;
use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::core::parser::Symbol;
use crate::core::version::AnyVersion;
use crate::core::version::Version;
use crate::core::vhdl::symbol::Architecture;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::vhdl::token::{Identifier, IdentifierError};
use crate::core::pkgid::{PkgIdError, PkgId};
use crate::util::anyerror::{AnyError, Fault};

/// The complete V.L.N:IDENTIFIER to pinpoint a particular VHDL symbol.
#[derive(Debug, PartialEq)]
struct EntityPath {
    ip: Option<PkgId>,
    entity: Identifier,
}

impl std::str::FromStr for EntityPath {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((ip, ent)) = s.split_once(':') {
            Ok(Self {
                ip: { if ip.is_empty() { None } else { Some(PkgId::from_str(ip)?) } },
                entity: Identifier::from_str(ent)?,
            })
        } else {
            // require the ':' for consistency
            return Err(AnyError(format!("missing ':' separator")))?
        }
    }
}

impl From<IdentifierError> for AnyError {
    fn from(e: IdentifierError) -> Self { 
        AnyError(e.to_string())
    }
}

impl From<PkgIdError> for AnyError {
    fn from(e: PkgIdError) -> Self { 
        AnyError(e.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub struct Get {
    entity_path: EntityPath,
    signals: bool,
    component: bool,
    instance: bool,
    architectures: bool,
    version: Option<AnyVersion>,
    info: bool,
    peek: bool,
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
            version: cli.check_option(Optional::new("ver").switch('v'))?,
            info: cli.check_flag(Flag::new("info"))?,
            peek: cli.check_flag(Flag::new("peek"))?,
            name: cli.check_option(Optional::new("name").value("identifier"))?,
            entity_path: cli.require_positional(Positional::new("entity"))?,
        });
        command
    }
}

use crate::core::parser::Parse;
use crate::core::vhdl;
use crate::core::vhdl::symbol;
use crate::core::vhdl::token::VHDLTokenizer;

impl Command for Get {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {  
        // --name can only be used with --instance is set
        if self.name.is_some() && self.instance == false {
            return Err(AnyError(format!("'{}' can only be used with '{}'", "--name".yellow(), "--instance".yellow())))?
        }

        // must be in an IP if omitting the pkgid
        if self.entity_path.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }
            self.run(&IpManifest::from_path(c.get_ip_path().unwrap())?, true, None) // do not add self to requirements
        } else {
            // gather the catalog (all manifests)
            let mut catalog = Catalog::new()
                .store(c.get_store_path())
                .development(c.get_development_path().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?;
            let ids = catalog.inner().keys().map(|f| { f }).collect();
            let target = crate::core::ip::find_ip(&self.entity_path.ip.as_ref().unwrap(), ids)?;
            
            // find all manifests and prioritize installed manifests over others but to help with errors/confusion
            let status = catalog.inner_mut().remove(&target).unwrap();

            // determine version to grab
            let v = self.version.as_ref().unwrap_or(&AnyVersion::Latest);
            let ip = match v {
                AnyVersion::Dev => status.get_dev(),
                _ => status.get_install(v)
            };
            if ip.is_none() {
                // check if the ip is available for more helpful error message
                return match status.get_available(v) {
                    Some(_) => Err(AnyError(format!("ip '{}' is not installed but is available; if you want to use any unit from it try installing the ip", target))),
                    None => Err(AnyError(format!("ip '{}' is not found as version '{}'", target, v)))
                }?
            }

            let current_ip = match c.goto_ip_path() {
                Ok(_) => Some(IpManifest::from_path(c.get_ip_path().unwrap())?),
                Err(_) => None,
            };
            
            self.run(ip.unwrap(), false, if self.peek == true { None } else { current_ip })
        }
    }
}

impl Get {
    fn run(&self, ip: &IpManifest, is_self: bool, current_ip: Option<IpManifest>) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let ent = match Self::fetch_entity(&self.entity_path.entity, &ip) {
            Ok(r) => r,
            Err(e) => return Err(GetError::SuggestProbe(e.to_string(), ip.get_pkgid().clone(), self.version.as_ref().unwrap_or(&AnyVersion::Latest).clone()))?
        };

        // add to dependency list if within a ip
        if let Some(mut cur_ip) = current_ip {
            cur_ip.insert_dependency(ip.get_pkgid().clone(), self.version.as_ref().unwrap_or(&AnyVersion::Latest).clone());
            cur_ip.get_manifest_mut().save()?;
        }

        // display architectures    
        if self.architectures == true {
            println!("{}", ent.get_architectures());
        }

        // display component declaration
        if self.component == true {
            println!("{}", ent.into_component());
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
        // make the library reference the current working ip 'work' if its internal
        let lib = match is_self {
            true => Some(String::from("work")),
            false => Some(ip.get_pkgid().get_library().as_ref().unwrap().to_string().replace("-", "_"))
        };
        // only display the direct entity instantiation code if not providing component code
        let lib = match self.component {
            true => None,
            false => lib
        };

        // display instantiation code
        if self.instance == true {
            let name = match &self.name {
                Some(iden) => iden.to_string(),
                None => "uX".to_string(),
            };
            println!("{}", ent.into_instance(&name, lib));
        }

        Ok(())
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &Identifier, ip: &IpManifest) -> Result<symbol::Entity, Fault> {
        let files = crate::util::filesystem::gather_current_files(&ip.get_root());
        for f in files {
            // lex and parse
            if crate::core::fileset::is_vhdl(&f) == true {
                let text = std::fs::read_to_string(f)?;
                // store list of all architectures while iterating through all symbols
                let mut architectures: Vec<Architecture> = Vec::new();
                // pull all architectures
                let units: Vec<Symbol<symbol::VHDLSymbol>> = vhdl::symbol::VHDLParser::parse(VHDLTokenizer::from_source_code(&text).into_tokens())
                    .into_iter()
                    .filter_map(|f| if f.is_ok() { 
                        let unit = f.unwrap();
                        match unit.as_ref().as_architecture() {
                            Some(_) => {
                                architectures.push(unit.take().into_architecture().unwrap());
                                None 
                            },
                            None => Some(unit)
                        }
                    } else { None }
                    ).collect();

                // detect entity
                let req_ent = units
                    .into_iter() 
                    .filter_map(|r| r.take().into_entity())
                    .find(|p| p.get_name() == iden);

                if let Some(mut entity) = req_ent {
                    // find all architectures that match entity name/owner
                    architectures.into_iter()
                        .for_each(|arch| {
                            if arch.entity() == entity.get_name() { entity.link_architecture(arch); }
                        });
                    return Ok(entity)
                }
            }
        }
        Err(GetError::EntityNotFound(iden.clone(), ip.get_pkgid().clone(), ip.get_version().clone()))?
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
Quick help sentence about command.

Usage:
    orbit get [options] <entity-path>

Args:
    <entity-path>       pkgid and entity identifier

Options:
    --ver, -v <version> ip version to use
    --component, -c     print component declaration
    --signals,   -s     print signal declarations
    --instance,  -i     print instantation
    --info              access code file's header comment
    --architecture, -a  print available architectures
    --peek              do not add the ip to the dependency table
    --name <identifier> specific instance identifier

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