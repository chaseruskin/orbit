use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional};
use crate::interface::errors::CliError;
use crate::core::context::Context;

#[derive(Debug, PartialEq)]
pub struct Plan {
    plugin: Option<String>,
}

impl Command for Plan {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // check that user is in an IP directory
        match c.get_ip_path() {
            Some(cwd) => {
                // set the current working directory to here
                std::env::set_current_dir(&cwd).expect("could not change directories");
            }
            None => {
                panic!("must be called from an IP's directory");
            }
        }
        // :todo: pass in the current IP struct, and the filesets to gather
        Ok(self.run(c.get_build_dir()))
    }
}

impl Plan {
    fn run(&self, build_dir: &str) -> () {
        let mut blueprint_path = std::env::current_dir().unwrap();
        // :todo: gather filesets

        // create a single-level directory (ok if it already exists)
        if std::path::PathBuf::from(build_dir).exists() == false {
            std::fs::create_dir(build_dir).expect("could not create build dir");
        }
        blueprint_path.push(build_dir);
        blueprint_path.push("blueprint.tsv");
        std::fs::File::create(&blueprint_path).expect("could not create blueprint file");
        // create a blueprint file
        println!("info: Blueprint created at: {}", blueprint_path.display());
    }
}

impl FromCli for Plan {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(PLAN);
        let command = Ok(Plan {
            plugin: cli.check_positional(Positional::new("plugin"))?,
        });
        command
    }
}

const PLAN: &str = "\
Generates a blueprint file.

Usage:
    orbit plan [<plugin>]

Args:
    <plugin>                collect filesets defined for this plugin

Options:
    --fileset <key=pattern> set an additional fileset
    --build-dir <dir>       set the output build directory

Use 'orbit help plan' to learn more about the command.
";