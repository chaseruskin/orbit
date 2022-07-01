use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional};
use crate::interface::errors::CliError;
use crate::core::pkgid;
use crate::interface::arg::Arg;
use crate::core::context::Context;
use std::error::Error;
use crate::util::anyerror::AnyError;
use crate::core::ip::Ip;
use crate::commands::search::Search;
use crate::core::template::Template;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct New {
    ip: pkgid::PkgId,
    rel_path: Option<std::path::PathBuf>,
    template: Option<String>,
}

impl FromCli for New {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(New {
            ip: cli.require_positional(Positional::new("ip"))?,
            rel_path: cli.check_option(Optional::new("path"))?,
            template: cli.check_option(Optional::new("template").value("alias"))?,
        });
        command
    }
}

impl Command for New {
    type Err = Box<dyn Error>;
    fn exec(&self, context: &Context) -> Result<(), Self::Err> {
        // extra validation for a new IP spec to contain all fields (V.L.N)
        if let Err(e) = self.ip.fully_qualified() {
            return Err(CliError::BadType(Arg::Positional(Positional::new("ip")), e.to_string()))?
        }
        let root = context.get_development_path().unwrap();

        // verify the pkgid is not taken
        let ips = Search::all_pkgid(
            (context.get_development_path().unwrap(), 
            context.get_cache_path(), 
            &context.get_vendor_path()))?;
        if ips.contains_key(&self.ip) == true {
            return Err(AnyError(format!("ip pkgid '{}' already taken", self.ip)))?
        }

        // verify the template exists
        let template = if let Some(alias) = &self.template {
            match context.get_templates().get(alias) {
                Some(t) => Some(t),
                None => return Err(AnyError(format!("template '{}' does not exist", alias)))?
            }
        } else {
            None
        };

        // only pass in necessary variables from context
        self.run(root, context.force, template)
    }
}

impl New {
    fn run(&self, root: &std::path::PathBuf, force: bool, template: Option<&Template>) -> Result<(), Box<dyn Error>> {
        // create ip stemming from ORBIT_PATH with default /VENDOR/LIBRARY/NAME
        let ip_path = if self.rel_path.is_none() {
            root.join(self.ip.get_vendor().as_ref().unwrap())
                .join(self.ip.get_library().as_ref().unwrap())
                .join(self.ip.get_name())
        } else {
            root.join(self.rel_path.as_ref().unwrap())
        };

        // verify the ip would exist alone on this path (cannot nest IPs)
        {
            // go to the very tip existing component of the path specified
            let mut path_clone = ip_path.clone();
            while path_clone.exists() == false {
                path_clone.pop();
            }
            // verify there are no current IPs living on this path
            if let Some(other_path) = Context::find_ip_path(&path_clone) {
                return Err(AnyError(format!("an IP already exists at path {}", other_path.display())))?
            }
        }

        let ip = Ip::new(ip_path, force)?.create_manifest(&self.ip)?;

        // import template if found
        if let Some(t) = template {
            // create hashmap to store variables
            let mut vars = HashMap::new();
            vars.insert("orbit.ip.name".to_owned(), self.ip.get_name().to_string());
            vars.insert("orbit.ip.library".to_owned(), self.ip.get_library().as_ref().unwrap().to_string());
            vars.insert("orbit.ip.vendor".to_owned(), self.ip.get_library().as_ref().unwrap().to_string());
            vars.insert("orbit.ip".to_owned(), self.ip.to_string());
            t.import(ip.get_path(), &vars)?;
        }
        
        println!("info: new ip created at {}", ip.get_path().display());
        Ok(())
    }
}

const HELP: &str = "\
Create a new orbit ip package.

Usage:
    orbit new [options] <ip>

Args:
    <ip>                the V.L.N for the new package (pkgid)

Options:
    --path <path>       set the destination directory
    --template <alias>  specify a template to import

Use 'orbit help new' to read more about the command.
";