use crate::core::manifest::IpManifest;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Optional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;
use crate::core::plugin::Plugin;
use crate::util::environment;
use crate::util::environment::ORBIT_BLUEPRINT;

#[derive(Debug, PartialEq)]
pub struct Build {
    alias: Option<String>,
    list: bool,
    command: Option<String>,
    args: Vec<String>,
    verbose: bool,
}

impl FromCli for Build {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Build {
            alias: cli.check_option(Optional::new("plugin").value("alias"))?,
            list: cli.check_flag(Flag::new("list"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            command: cli.check_option(Optional::new("command").value("cmd"))?,
            args: cli.check_remainder()?,
        });
        command
    }
}

impl Command for Build {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // display plugin list and exit
        if self.list == true {
            println!("{}", Plugin::list_plugins(&c.get_plugins().values().into_iter().collect::<Vec<&Plugin>>()));
            return Ok(())
        }

        // verify only 1 option is provided
        if self.command.is_some() && self.alias.is_some() {
            return Err(AnyError(format!("cannot execute both a plugin and command")))?
        }
        // verify running from an IP directory
        c.goto_ip_path()?;
        // verify a blueprint file exists at build directory
        let blueprint_file = c.get_ip_path().unwrap().join(c.get_build_dir()).join(BLUEPRINT_FILE);
        if blueprint_file.exists() == false {
            return Err(Box::new(AnyError(format!("no blueprint file to build from; consider running 'orbit plan'"))))
        }

        Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&IpManifest::from_path(c.get_ip_path().unwrap())?)?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE))
            .initialize();

        // load from .env file
        let envs = Environment::new()
            .from_env_file(&c.get_ip_path().unwrap().join(c.get_build_dir()))?;

        

        // check if ORBIT_PLUGIN was set and no command option was set
        let alias = match &self.alias {
            Some(n) => Some(n.as_str()),
            None => {
                if let Some(plug) = envs.get(environment::ORBIT_PLUGIN) {
                    // verify there was no command option to override default plugin call
                    if self.command.is_none() { Some(plug.get_value()) } else { None }
                } else {
                    None
                }
            }
        };
        // try to find plugin matching `command` name under the `alias`
        let plug = if let Some(a) = alias {
            match c.get_plugins().get(a) {
                Some(p) => Some(p),
                None => return Err(AnyError(format!("no plugin named '{}'", a)))?,
            }
        } else {
            None
        };

        envs.initialize();

        if plug.is_none() && self.command.is_none() {
            return Err(AnyError(format!("pass a plugin or a command for building")))?
        }

        self.run(plug)
    }
}

use std::process::Stdio;

use super::plan::BLUEPRINT_FILE;

impl Build {
    fn run(&self, plug: Option<&Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args, self.verbose)
        } else if let Some(cmd) = &self.command {
            if self.verbose == true {
                let s = self.args.iter().fold(String::new(), |x, y| { x + "\"" + &y + "\" " });
                println!("running: {} {}", cmd, s);
            }
            let mut proc = std::process::Command::new(cmd)
                .args(&self.args)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            let exit_code = proc.wait()?;
            match exit_code.code() {
                Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { Ok(()) },
                None =>  Err(AnyError(format!("terminated by signal")))?
            }
        } else {
            Ok(())
        }
    }
}

const HELP: &str = "\
Execute a backend tool/workflow.

Usage:
    orbit build [options] [--] [args]...

Options:
    --plugin <alias>    plugin to execute
    --command <cmd>     command to execute
    --list              view available plugins
    --verbose           display the command being executed
    -- args...          arguments to pass to the requested command

Use 'orbit help build' to learn more about the command.
";