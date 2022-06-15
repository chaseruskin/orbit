use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::vhdl::vhdl::{Identifier, IdentifierError};
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
                ip: Some(PkgId::from_str(ip)?),
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
}

impl FromCli for Get {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Get {
            signals: cli.check_flag(Flag::new("signals").switch('s'))?,
            component: cli.check_flag(Flag::new("component").switch('c'))?,
            instance: cli.check_flag(Flag::new("instance").switch('i'))?,
            entity_path: cli.require_positional(Positional::new("entity"))?,
        });
        command
    }
}

impl Command for Get {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, _: &Context) -> Result<(), Self::Err> {
        self.run()
    }
}

impl Get {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

const HELP: &str = "\
Quick help sentence about command.

Usage:
    orbit get [options] <entity>

Args:
    <entity>            entity path 

Options:
    --component, -c     print component declaration
    --signals,   -s     print signal declarations
    --instance,  -i     print instantation

Use 'orbit help get' to learn more about the command.
";