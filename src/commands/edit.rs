use std::path::PathBuf;
use std::str::FromStr;

use clif::cmd::{FromCli, Command};
use crate::core::catalog::Catalog;
use crate::core::config::CONFIG_FILE;
use crate::core::config::Config;
use crate::core::extgit::ExtGitError;
use clif::Cli;
use clif::arg::{Flag, Optional};
use clif::Error as CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::AnyError;


#[derive(Debug, PartialEq)]
pub enum EditMode {
    Open,
    Path
}

impl FromStr for EditMode {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "path" => Ok(Self::Path),
            _ => Err(AnyError(format!("unsupported mode")))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Edit {
    editor: Option<String>,
    ip: Option<PkgId>,
    config: bool,
    mode: EditMode,
}

impl FromCli for Edit {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Edit {
            mode: cli.check_option(Optional::new("mode"))?.unwrap_or(EditMode::Open),
            config: cli.check_flag(Flag::new("config"))?,
            editor: cli.check_option(Optional::new("editor"))?,
            ip: cli.check_option(Optional::new("ip").value("pkgid"))?,
        });
        command
    }
}

impl Command<Context> for Edit {
    type Status = Result<(), Box<dyn std::error::Error>>;

    fn exec(&self, c: &Context) -> Self::Status {
        let sel_editor = Self::configure_editor(&self.editor, &c.get_config())?;
        // open global configuration file
        if self.config == true {
            let config_path = c.get_config().get_root().join(CONFIG_FILE);
            return match &self.mode {
                EditMode::Open => Edit::invoke(&sel_editor, &config_path),
                EditMode::Path => { println!("{}", crate::util::filesystem::normalize_path(config_path).display()); Ok(()) }
            }
        // open an ip
        } else if self.ip.is_some() == true {
            // collect manifest from DEV_PATH
            let catalog = Catalog::new()
                .development(c.get_development_path().as_ref().unwrap())?
                .installations(c.get_cache_path())?
                .available(c.get_vendors())?
                .store(c.get_store_path());
            self.run(&catalog, &sel_editor)
        } else {
            panic!("nothing to edit")
        }
    }
}

use crate::core::ip;
use crate::util::anyerror::Fault;

impl Edit {
    /// Determines the editor from a priority list. If option 1 or 3 is chosen,
    /// then it will attempt to resolve the relative path if it is a relative path.
    /// 
    /// Priority 
    /// 1. command-line argument `arg`
    /// 2. environment variable EDITOR
    /// 3. configuration value in config.toml for `core.editor`
    /// 
    /// Errors if no editor can be returned.
    pub fn configure_editor(arg: &Option<String>, config: &Config) -> Result<String, Fault> {
        match &arg {
            // prioritize the command-line argument as overriding a default value
            Some(e) => Ok(crate::util::filesystem::resolve_rel_path(&std::env::current_dir().unwrap(), e)),
            None => {
                if let Ok(val) = std::env::var("EDITOR") {
                   Ok(val)
                } else {
                    // try the config.toml
                    match config.get_as_str("core", "editor")? {
                        // try to resolve relative path
                        Some(e) => Ok(crate::util::filesystem::resolve_rel_path(config.get_root(), e)),
                        None => Err(AnyError("no editor detected".to_owned()))?
                    }
                }
            }
        }
    }

    /// Calls a command `editor` with the argument `path`.
    /// 
    /// Silently captures the output and only prints stderr if the command failed with non-zero
    /// exit code.
    pub fn invoke(editor: &str, path: &PathBuf) -> Result<(), Fault> {
        // perform the process
        let output = std::process::Command::new(editor)
            .arg(path)
            .output()?;
        match output.status.code() {
            Some(num) => if num != 0 { Err(ExtGitError::NonZeroCode(num, output.stderr))? } else { Ok(()) },
            None => Err(ExtGitError::SigTermination)?,
        }
    }

    fn run(&self, catalog: &Catalog, editor: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ids: Vec<&PkgId> = catalog.inner().keys().map(|f| f).collect();
        // find the full ip name among the manifests to get the path
        let result = ip::find_ip(&self.ip.as_ref().unwrap(), ids)?;

        let ip = match catalog.inner().get(&result).unwrap().get_dev() {
            // perform the process
           Some(ip) => ip,
           None => return Err(AnyError(format!("ip '{}' is not found on the DEV_PATH", result)))?    
        };
        match &self.mode {
            EditMode::Open => {
                Self::invoke(editor, &ip.get_root())
            }
            EditMode::Path => {
                println!("{}", crate::util::filesystem::normalize_path(ip.get_root()).display());
                Ok(())
            }
        }
    }
}

const HELP: &str = "\
Open a text editor to develop an ip or orbit-related files.
Usage:
    orbit edit [options]
Options:
    --ip <pkgid>       ip to open in development state
    --editor <cmd>     the command to call a text-editor
    --mode <mode>      select how to edit: 'open' or 'path'
    --config           modify the global configuration file
Use 'orbit help edit' to learn more about the command.
";