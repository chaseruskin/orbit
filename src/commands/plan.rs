use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::Optional;
use crate::interface::errors::CliError;
use crate::core::context::Context;
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct Plan {
    plugin: Option<String>,
    bench: Option<String>,
    top: Option<String>,
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
        // set top-level environment variables (:todo: verify these are valid toplevels to be set!)
        if let Some(t) = &self.top {
            std::env::set_var("ORBIT_TOP", t);
        }
        if let Some(b) = &self.bench {
            std::env::set_var("ORBIT_BENCH", b);
        }
        // :todo: pass in the current IP struct, and the filesets to gather
        Ok(self.run(c.get_build_dir()))
    }
}

impl Plan {
    fn run(&self, build_dir: &str) -> () {
        let mut blueprint_path = std::env::current_dir().unwrap();
        // gather filesets
        let files = crate::core::fileset::gather_current_files();
        // :todo: remove and properly include only-necessary in-order hdl files
        let vhdl_rtl_files = crate::core::fileset::collect_vhdl_files(&files, false);
        let vhdl_sim_files = crate::core::fileset::collect_vhdl_files(&files, true);
        // store data in blueprint TSV format
        let mut blueprint_data = String::new();
        for f in vhdl_rtl_files {
            blueprint_data += &format!("VHDL-RTL\twork\t{}\n", f);
        }
        for f in vhdl_sim_files {
            blueprint_data += &format!("VHDL-SIM\twork\t{}\n", f);
        }
        // create a single-level directory (ok if it already exists)
        if std::path::PathBuf::from(build_dir).exists() == false {
            std::fs::create_dir(build_dir).expect("could not create build dir");
        }
        // create the file
        blueprint_path.push(build_dir);
        blueprint_path.push("blueprint.tsv");
        let mut blueprint_file = std::fs::File::create(&blueprint_path).expect("could not create blueprint file");
        // write the data
        blueprint_file.write_all(blueprint_data.as_bytes()).expect("failed to write data to blueprint");
        // create a blueprint file
        println!("info: Blueprint created at: {}", blueprint_path.display());
    }
}

impl FromCli for Plan {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(PLAN);
        let command = Ok(Plan {
            top: cli.check_option(Optional::new("top"))?,
            bench: cli.check_option(Optional::new("bench"))?,
            plugin: cli.check_option(Optional::new("plugin"))?,
        });
        command
    }
}

const PLAN: &str = "\
Generates a blueprint file.

Usage:
    orbit plan [options]              

Options:
    --top <unit>            override auto-detected toplevel entity
    --bench <tb>            override auto-detected toplevel testbench
    --plugin <plugin>       collect filesets defined for this plugin
    --build-dir <dir>       set the output build directory
    --fileset <key=glob>... set an additional fileset
    --all                   include all found HDL files

Use 'orbit help plan' to learn more about the command.
";