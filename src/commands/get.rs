use colored::Colorize;

use crate::Command;
use crate::FromCli;
use crate::core::ip::Ip;
use crate::core::version::AnyVersion;
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
            entity_path: cli.require_positional(Positional::new("entity"))?,
        });
        command
    }
}

use crate::core::parser::Parse;
use crate::core::vhdl;
use crate::core::vhdl::symbol;
use crate::core::vhdl::token::VHDLTokenizer;
use crate::commands::search::Search;

impl Command for Get {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // must be in an IP if omitting the pkgid
        let (ip, is_self) = if self.entity_path.ip.is_none() {
            c.goto_ip_path()?;
            
            // error if a version is specified and its referencing the self IP
            if self.version.is_some() {
                return Err(AnyError(format!("cannot specify a version '{}' when referencing the current ip", "--ver".yellow())))?
            }
            (Ip::init_from_path(c.get_ip_path().unwrap().clone())?, true)
        } else {
            // grab installed ip
            let mut universe = Search::all_pkgid((c.get_development_path().unwrap(), c.get_cache_path(), &c.get_vendor_path()))?;
            let target = crate::core::ip::find_ip(&self.entity_path.ip.as_ref().unwrap(), universe.keys().into_iter().collect())?;
            
            // find all manifests and prioritize installed manifests over others but to help with errors/confusion
            let inventory = universe.remove(&target).unwrap().1;

            // @TODO determine version to grab
            let v = self.version.as_ref().unwrap_or(&AnyVersion::Latest);
            (crate::commands::probe::select_ip_from_version(&target, &v, inventory)?, false)
        };
        
        self.run(ip, is_self)
    }
}

impl Get {
    fn run(&self, ip: Ip, is_self: bool) -> Result<(), Fault> {
        // collect all hdl files and parse them
        let ent = Self::fetch_entity(&self.entity_path.entity, &ip)?;

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
            false => Some(ip.get_manifest().as_pkgid().get_library().as_ref().unwrap().to_string().replace("-", "_"))
        };
        // only display the direct entity instantiation code if not providing component code
        let lib = match self.component {
            true => None,
            false => lib
        };

        // display instantiation code
        if self.instance == true {
            println!("{}", ent.into_instance("uX", lib));
        }

        Ok(())
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &Identifier, ip: &Ip) -> Result<symbol::Entity, Box<dyn std::error::Error>> {
        let files = crate::core::fileset::gather_current_files(ip.get_path());
        for f in files {
            // lex and parse
            if crate::core::fileset::is_vhdl(&f) == true {
                let text = std::fs::read_to_string(f)?;
                let req_ent: Option<symbol::Entity> = vhdl::symbol::VHDLParser::parse(VHDLTokenizer::from_source_code(&text).into_tokens())
                    .into_iter()
                    .filter_map(|r| if r.is_ok() { r.unwrap().take().into_entity() } else { None })
                    .find(|p| p.get_name() == iden);
                if let Some(e) = req_ent {
                    return Ok(e)
                }
            }
        }
        Err(AnyError(format!("entity '{}' is not found in ip '{}'", iden, ip.get_manifest().as_pkgid())))?
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