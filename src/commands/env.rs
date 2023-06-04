use std::path::PathBuf;

use clif::cmd::{FromCli, Command};
use crate::core::v2::ip::Ip;
use clif::Cli;
use clif::arg::Positional;
use clif::Error as CliError;
use crate::core::context::Context;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::environment::ORBIT_BLUEPRINT;
use crate::util::environment::ORBIT_WIN_LITERAL_CMD;
use crate::util::filesystem::Standardize;
use crate::OrbitResult;
use crate::commands::plan::BLUEPRINT_FILE;

#[derive(Debug, PartialEq)]
pub struct Env {
    keys: Vec<String>,
}

impl FromCli for Env {
    fn from_cli(cli: &mut Cli) -> Result<Self,  CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        // collect all positional arguments
        let mut keys: Vec<String> = Vec::new();
        while let Some(c) = cli.check_positional(Positional::new("key"))? {
            keys.push(c);
        }
        let command = Ok(Env {
            keys: keys,
        });
        command
    }
}

impl Command<Context> for Env {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // assemble environment information
        let mut env = Environment::from_vec(vec![
            // @todo: context should own an `Environment` struct instead of this data transformation
            EnvVar::new().key(environment::ORBIT_HOME).value(PathBuf::standardize(c.get_home_path()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_CACHE).value(PathBuf::standardize(c.get_cache_path()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_QUEUE).value(PathBuf::standardize(c.get_queue_path()).to_str().unwrap()),
            EnvVar::new().key(environment::ORBIT_BUILD_DIR).value(c.get_build_dir()),
            EnvVar::new().key(environment::ORBIT_IP_PATH).value(PathBuf::standardize(c.get_ip_path().unwrap_or(&PathBuf::new())).to_str().unwrap()),
            EnvVar::new().key("EDITOR").value(&std::env::var("EDITOR").unwrap_or(String::new())),
            EnvVar::new().key("NO_COLOR").value(&std::env::var("NO_COLOR").unwrap_or(String::new())),
            ])
            .from_config(c.get_config())?
            .add(EnvVar::new().key(ORBIT_BLUEPRINT).value(BLUEPRINT_FILE));

        // add platform-specific environment variables
        if cfg!(target_os = "windows") {
            env = env.add(EnvVar::new().key(ORBIT_WIN_LITERAL_CMD).value(&std::env::var(ORBIT_WIN_LITERAL_CMD).unwrap_or(String::new())));
        }

        // check if in an ip to add those variables
        if let Some(ip_path) = c.get_ip_path() {
            // check ip
            if let Ok(ip) = Ip::load(ip_path.clone()) {
                env = env.from_ip(&ip)?;
            }
            // check the build directory
            env = env.from_env_file(&std::path::PathBuf::from(c.get_build_dir()))?;
        }
        
        self.run(env)
    }
}

impl Env {
    fn run(&self, env: Environment) -> Result<(), Box<dyn std::error::Error>> {
        let mut result = String::new();

        match self.keys.is_empty() {
            // print debugging output (all variables)
            true => {
                env.iter().for_each(|e| {
                    if result.is_empty() == false {
                        result.push('\n');
                    }
                    result.push_str(&format!("{:?}", e))
                });
            },
            false => {
                let mut initial = true;
                // print values only
                self.keys.iter().for_each(|k| {
                    if initial == false {
                        result.push('\n');
                    }
                    if let Some(entry) = env.get(k) {
                        result.push_str(&entry.get_value());
                    }
                    initial = false;
                });
            }
        }

        println!("{}", result);
        Ok(())
    }
}

const HELP: &str = "\
Display Orbit environment information.

Usage:
    orbit env [options]

Options:
    <key>...     Lookup environment variable to display its value

Use 'orbit help env' to learn more about the command.
";