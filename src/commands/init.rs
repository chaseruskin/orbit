use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional, Arg};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgId;
use crate::commands::search::Search;

#[derive(Debug, PartialEq)]
pub struct Init {
    ip: PkgId,
    repo: Option<String>,
    path: Option<std::path::PathBuf>,
}

impl FromCli for Init {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Init {
            repo: cli.check_option(Optional::new("git").value("repo"))?,
            path: cli.check_option(Optional::new("path"))?,
            ip: cli.require_positional(Positional::new(""))?,
        });
        command
    }
}

impl Command for Init {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // extra validation for a new IP spec to contain all fields (V.L.N)
        if let Err(e) = self.ip.fully_qualified() {
            return Err(Box::new(CliError::BadType(Arg::Positional(Positional::new("ip")), e.to_string())));
        }

        // verify the pkgid is not taken
        let ips = Search::all_pkgid(
            c.get_development_path().unwrap(), 
            c.get_cache_path(), 
            &c.get_vendor_path())?;
        if ips.contains(&self.ip) == true {
            return Err(AnyError(format!("ip pkgid '{}' already taken", self.ip)))?
        }

        // get dev path join with options
        let path = c.get_development_path().unwrap();

        self.run()
    }
}

impl Init {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

const HELP: &str = "\
Initialize a new ip from an existing project.

Usage:
    orbit init [options] <ip>

Args:
    <ip>                the pkgid to label the existing project

Options:
    --git <repo>        repository to clone
    --path <path>       destination path to initialize 

Use 'orbit help init' to learn more about the command.
";