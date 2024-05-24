use std::path::PathBuf;

use super::plan::BLUEPRINT_FILE;
use crate::commands::helps::build;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::plugin::Plugin;
use crate::core::plugin::PluginError;
use crate::core::plugin::Process;
use crate::util::anyerror::AnyError;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_BLUEPRINT;
use crate::util::environment::ORBIT_BUILD_DIR;

use cliproc::{cli, proc};
use cliproc::{Cli, Flag, Help, Optional, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Build {
    alias: Option<String>,
    list: bool,
    force: bool,
    command: Option<String>,
    build_dir: Option<String>,
    args: Vec<String>,
    verbose: bool,
}

impl Subcommand<Context> for Build {
    fn construct<'c>(cli: &'c mut Cli) -> cli::Result<Self> {
        cli.check_help(Help::default().text(build::HELP))?;
        Ok(Build {
            // Flags
            list: cli.check_flag(Flag::new("list"))?,
            verbose: cli.check_flag(Flag::new("verbose"))?,
            force: cli.check_flag(Flag::new("force"))?,
            // Options
            alias: cli.check_option(Optional::new("plugin").value("alias"))?,
            build_dir: cli.check_option(Optional::new("build-dir").value("dir"))?,
            command: cli.check_option(Optional::new("command").value("cmd"))?,
            // Remaining args
            args: cli.check_remainder()?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // try to find plugin matching `command` name under the `alias`
        let plug = if let Some(name) = &self.alias {
            match c.get_config().get_plugins().get(name.as_str()) {
                Some(&p) => Some(p),
                None => return Err(PluginError::Missing(name.to_string()))?,
            }
        } else {
            None
        };
        // display plugin list and exit
        if self.list == true {
            match plug {
                Some(plg) => println!("{}", plg),
                None => println!(
                    "{}",
                    Plugin::list_plugins(
                        &mut c
                            .get_config()
                            .get_plugins()
                            .values()
                            .into_iter()
                            .collect::<Vec<&&Plugin>>()
                    )
                ),
            }
            return Ok(());
        }

        // verify only 1 option is provided
        if self.command.is_some() && self.alias.is_some() {
            return Err(AnyError(format!(
                "Cannot execute both a plugin and command"
            )))?;
        }
        // verify running from an IP directory and enter IP's root directory
        c.goto_ip_path()?;

        // determine the build directory based on cli priority
        let default_build_dir = c.get_build_dir();
        let b_dir = self.build_dir.as_ref().unwrap_or(&default_build_dir);

        // todo: is this necessary? -> no, but maybe add a flag/option to bypass (and also allow plugins to specify if they require blueprint in settings)
        // idea: [[plugin]] require-plan = false
        // assert a blueprint file exists in the specified build directory
        if c.get_ip_path()
            .unwrap()
            .join(b_dir)
            .join(BLUEPRINT_FILE)
            .exists()
            == false
            && self.force == false
        {
            return Err(AnyError(format!("No blueprint file to build from in directory '{}'\n\nTry `orbit plan --build-dir {0}` to generate a blueprint file", b_dir)))?;
        }

        Environment::new()
            // read config.toml for setting any env variables
            .from_config(c.get_config())?
            // read ip manifest for env variables
            .from_ip(&Ip::load(c.get_ip_path().unwrap().clone(), true)?)?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE))
            .add(EnvVar::new().key(ORBIT_BUILD_DIR).value(b_dir))
            .initialize();

        // load from .env file from the correct build dir
        let envs = match Environment::new().from_env_file(&c.get_ip_path().unwrap().join(b_dir)) {
            Ok(r) => r,
            Err(e) => match self.force {
                false => return Err(AnyError(format!("Failed to read .env file: {}", e)))?,
                true => Environment::new(),
            },
        };

        // check if ORBIT_PLUGIN was set and no command option was set
        let plug = match plug {
            // already configured from the command-line
            Some(plg) => Some(plg),
            // was not set on the command-line
            None => {
                if let Some(plug) = envs.get(environment::ORBIT_PLUGIN) {
                    // verify there was no command option to override default plugin call
                    if self.command.is_none() {
                        match c.get_config().get_plugins().get(plug.get_value()) {
                            Some(&p) => Some(p),
                            None => {
                                return Err(PluginError::Missing(plug.get_value().to_string()))?
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };

        envs.initialize();

        if plug.is_none() && self.command.is_none() {
            return Err(AnyError(format!(
                "Building requires a plugin or a command to process"
            )))?;
        }

        // make sure the directory exists
        if PathBuf::from(&b_dir).exists() == false {
            std::fs::create_dir(&b_dir)?;
        }

        // start command from the build directory
        self.run(plug, &b_dir)
    }
}

impl Build {
    fn run(&self, plug: Option<&Plugin>, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        // if there is a match run with the plugin then run it
        if let Some(p) = plug {
            p.execute(&self.args, self.verbose, dir)
        } else if let Some(cmd) = &self.command {
            if self.verbose == true {
                let s = self
                    .args
                    .iter()
                    .fold(String::new(), |x, y| x + "\"" + &y + "\" ");
                println!("info: Running: {} {}", cmd, s);
            }
            let mut proc = crate::util::filesystem::invoke(
                dir,
                cmd,
                &self.args,
                Context::enable_windows_bat_file_match(),
            )?;
            let exit_code = proc.wait()?;
            match exit_code.code() {
                Some(num) => {
                    if num != 0 {
                        Err(AnyError(format!("Exited with error code: {}", num)))?
                    } else {
                        Ok(())
                    }
                }
                None => Err(AnyError(format!("Terminated by signal")))?,
            }
        } else {
            Ok(())
        }
    }
}
