use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional, Arg};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgId;
use crate::commands::search::Search;
use crate::core::ip::Ip;
use crate::core::extgit::ExtGit;

#[derive(Debug, PartialEq)]
pub struct Init {
    ip: PkgId,
    repo: Option<String>,
    rel_path: Option<std::path::PathBuf>,
}

impl FromCli for Init {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Init {
            repo: cli.check_option(Optional::new("git").value("repo"))?,
            rel_path: cli.check_option(Optional::new("path"))?,
            ip: cli.require_positional(Positional::new("ip"))?,
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
        self.run(path, c.force)
    }
}

impl Init {
    fn run(&self, root: &std::path::PathBuf, _: bool) -> Result<(), Box<dyn std::error::Error>> {
        // create ip stemming from ORBIT_PATH with default /VENDOR/LIBRARY/NAME
        let ip_path = if self.rel_path.is_none() {
            root.join(self.ip.get_vendor().as_ref().unwrap())
                .join(self.ip.get_library().as_ref().unwrap())
                .join(self.ip.get_name())
        } else {
            root.join(self.rel_path.as_ref().unwrap())
        };

        if std::path::Path::exists(&ip_path) == true {
            return Err(AnyError(format!("failed to create new ip because directory '{}' already exists", ip_path.display())))?
        }

        // verify the ip would exist alone on this path (cannot nest IPs)
        {
            // go to the very tip existing component of the path specified
            let mut path_clone = ip_path.clone();
            while path_clone.exists() == false {
                path_clone.pop();
            }
            // verify there are no current IPs living on this path
            if let Some(other_path) = Context::find_ip_path(&path_clone) {
                return Err(Box::new(AnyError(format!("an IP already exists at path {}", other_path.display()))))
            }
        }

        // clone if given a git url
        if let Some(url) = &self.repo {
            ExtGit::new().command(None).clone(url, &ip_path)?;
        }

        // create a manifest at the ip path
        let ip = Ip::from_path(ip_path).create_manifest(&self.ip)?;

        // if there was a repository then add it as remote
        if let Some(url) = &self.repo {
            // must be remote link if not on filesystem
            if std::path::Path::exists(&std::path::PathBuf::from(url)) == false {
                // write 'repository' key
                let mut ip = ip.into_manifest();
                ip.0.write("ip", "repository", url);
                ip.0.save()?;
            }
        }
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