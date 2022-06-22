use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::vhdl::token::{Identifier, IdentifierError};
use crate::core::pkgid::PkgId;
use crate::util::anyerror::AnyError;

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
            Ok(Self {
                ip: None,
                entity: Identifier::from_str(s)?,
            })
        }
    }
}

impl From<IdentifierError> for AnyError {
    fn from(e: IdentifierError) -> Self { 
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
    info: bool,
    // --edition flag? to specify what version (because --version is taken)
}

impl FromCli for Get {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Get {
            signals: cli.check_flag(Flag::new("signals").switch('s'))?,
            component: cli.check_flag(Flag::new("component").switch('c'))?,
            instance: cli.check_flag(Flag::new("instance").switch('i'))?,
            architectures: cli.check_flag(Flag::new("architecture").switch('a'))?,
            info: cli.check_flag(Flag::new("info"))?,
            entity_path: cli.require_positional(Positional::new("entity"))?,
        });
        command
    }
}

use crate::core::ip;
use crate::core::manifest;
use crate::core::parser::Parse;
use crate::core::vhdl;
use crate::core::vhdl::symbol;
use crate::core::vhdl::token::VHDLTokenizer;

impl Command for Get {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // must be in an IP if omitting the pkgid
        let path = if self.entity_path.ip.is_none() {
            c.goto_ip_path()?;
            c.get_ip_path().unwrap().clone()
        } else {
            // grab installed ip
            let installed_ip = manifest::IpManifest::detect_all(c.get_cache_path())?;
            // find all manifests? and prioritize installed manifests over others but to help with errors/confusion
            let manifest = ip::find_ip(&self.entity_path.ip.as_ref().unwrap(), &installed_ip)?;
            // println!("{}", manifest.as_pkgid());
            manifest.0.get_path().parent().unwrap().to_path_buf()
        };
        // find the IP (@IDEA have flag to indicate if to use the in-dev version vs. cache?)
        // $ orbit get gates:nor_gate --edition latest --edition 1.0.0 --edition dev
        // get the directory where the IP lives
        // collect all hdl files and parse them
        let ent = Self::fetch_entity(&self.entity_path.entity, &path)?;

        println!("{:?}", ent);
        Ok(())
        // todo!("collect the hdl files for parsing");
        // get the VHDLSymbol list and find the entity matching the provided name
        // todo!("get the VHDL symbols and find matching entity");

        // self.run()
    }
}

impl Get {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    /// Parses through the vhdl files and returns a desired entity struct.
    fn fetch_entity(iden: &Identifier, ip_path: &std::path::PathBuf) -> Result<symbol::Entity, Box<dyn std::error::Error>> {
        let files = crate::core::fileset::gather_current_files(ip_path);
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
        panic!("entity '{}' does not exist in this ip", iden);
    }
}

const HELP: &str = "\
Quick help sentence about command.

Usage:
    orbit get [options] <entity-path>

Args:
    <entity-path>       pkgid and entity identifier

Options:
    --component, -c     print component declaration
    --signals,   -s     print signal declarations
    --instance,  -i     print instantation
    --info              access code file's header comment
    --architecture, -a  print available architectures

Use 'orbit help get' to learn more about the command.
";


#[cfg(test)]
mod test {
    use super::*;

    use std::str::FromStr;

    #[test]
    #[ignore]
    fn fetch_entity() {
        let _ = Get::fetch_entity(&Identifier::from_str("or_gate").unwrap(), &std::path::PathBuf::from("./test/data/gates")).unwrap();
        panic!("inspect")
    }
}