use colored::Colorize;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional, Arg};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::pkgid::PkgId;
use crate::core::extgit::ExtGit;
use crate::util::url::Url;

#[derive(Debug, PartialEq)]
pub struct Init {
    ip: PkgId,
    repo: Option<Url>,
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

        // verify only --path can be used with --git
        if self.rel_path.is_some() && self.repo.is_none() {
            return Err(AnyError(format!("'{}' can only be used with '{}'", "--path".yellow(), "--git".yellow())))?
        }

        // verify the pkgid is not taken
        {
            let catalog = Catalog::new()
                .development(c.get_development_path().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?;
            if catalog.inner().contains_key(&self.ip) == true {
                return Err(AnyError(format!("ip pkgid '{}' already taken", self.ip)))?
            }
        }

        let path = match &self.repo {
            None => std::env::current_dir()?,
            Some(_) => {
                match &self.rel_path {
                    Some(extra_path) => {
                        if extra_path.is_relative() {
                            c.get_development_path().unwrap().join(extra_path)
                        } else {
                            extra_path.to_owned()
                        }
                    },
                    None => {
                        c.get_development_path().unwrap()
                            .join(self.ip.get_vendor().as_ref().unwrap().as_ref())
                            .join(self.ip.get_library().as_ref().unwrap().as_ref())
                            .join(self.ip.get_name().as_ref())
                    },
                }
            }
        };
        self.run(path, c.force)
    }
}

impl Init {
    /// Initializes a project at an exising path.
    /// 
    /// Note the path must exist unless cloning from a git repository.
    fn run(&self, ip_path: std::path::PathBuf, _: bool) -> Result<(), Box<dyn std::error::Error>> {
        // the path must exist if not cloning from a repository
        if std::path::Path::exists(&ip_path) == false && self.repo.is_none() {
            return Err(AnyError(format!("failed to initialize ip because directory '{}' does not exist", ip_path.display())))?
        }

        // cannot clone into a non-empty directory
        if self.repo.is_some() && ip_path.is_dir() && std::fs::read_dir(&ip_path)?.count() > 0 {
            return Err(AnyError(format!("failed to initialize ip because directory '{}' is not empty to clone repository into", ip_path.display())))?
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
                return Err(Box::new(AnyError(format!("an ip already exists at path {}", other_path.display()))))
            }
        }

        // clone if given a git url
        if let Some(url) = &self.repo {
            ExtGit::new(None)
                .clone(url, &ip_path, false)?;
        }

        // create a manifest at the ip path
        let mut ip = IpManifest::create(ip_path, &self.ip, false, true)?;

        // if there was a repository then add it as remote
        if let Some(url) = &self.repo {
            // must be remote link if not on filesystem @TODO revisit with `Source` enum with `Path` variant
            // if std::path::Path::exists(&std::path::PathBuf::from(url)) == false {
                // write 'repository' key
                ip.get_manifest_mut().write("ip", "repository", url.to_string());
                ip.get_manifest_mut().save()?;
            // }
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