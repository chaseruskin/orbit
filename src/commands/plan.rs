use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::Optional;
use crate::interface::errors::CliError;
use crate::core::context::Context;
use std::ffi::OsString;
use std::io::Write;
use crate::core::fileset::Fileset;

#[derive(Debug, PartialEq)]
pub struct Plan {
    plugin: Option<String>,
    bench: Option<String>,
    top: Option<String>,
    build_dir: Option<String>,
    filesets: Option<Vec<Fileset>>
}

impl Command for Plan {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // check that user is in an IP directory
        c.goto_ip_path()?;
        // set top-level environment variables (@TODO verify these are valid toplevels to be set!)
        if let Some(t) = &self.top {
            std::env::set_var("ORBIT_TOP", t);
        }
        if let Some(b) = &self.bench {
            std::env::set_var("ORBIT_BENCH", b);
        }
        // determine the build directory
        let b_dir = if let Some(dir) = &self.build_dir {
            dir
        } else {
            c.get_build_dir()
        };
        // @TODO pass in the current IP struct
        Ok(self.run(b_dir))
    }
}

use crate::core::vhdl::parser;
use crate::util::graph::Graph;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
struct HashNode {
    index: usize,
    files: HashSet<String>,
}

impl HashNode {
    fn index(&self) -> usize {
        self.index
    }
    
    fn new(index: usize, file: String) -> Self {
        let mut set = HashSet::new();
        set.insert(file);
        Self {
            index: index,
            files: set,
        }
    }

    fn add_file(&mut self, file: String) {
        self.files.insert(file);
    }
}

impl Plan {
    fn run(&self, build_dir: &str) -> () {
        let mut build_path = std::env::current_dir().unwrap();
        build_path.push(build_dir);
        // gather filesets
        let files = crate::core::fileset::gather_current_files();

        let mut g = Graph::new();
        // entity identifier, HashNode
        let mut map = HashMap::<String, HashNode>::new();
        // read all files
        let mut archs = Vec::new();
        for f in &files {
            if f.ends_with(".vhd") == true {
                let contents = std::fs::read_to_string(f).unwrap();
                let symbols = parser::VHDLParser::read(&contents).into_symbols();
                // add all entities to a graph
                let mut iter = symbols.into_iter().filter_map(|f| {
                    match f {
                        parser::VHDLSymbol::Entity(_) => Some(f.get_iden().to_string()),
                        parser::VHDLSymbol::Architecture(arch) => {
                            archs.push(arch);
                            None
                        }
                        _ => None,
                    }
                });
                while let Some(e) = iter.next() {
                    let index = g.add_node();
                    map.insert(e.to_string(), HashNode::new(index, f.to_string()));
                }
            }
        }

        // go through all architectures and make the connections
        let mut archs = archs.into_iter();
        while let Some(arch) = archs.next() {
            // link to the owner
            // map.get_mut(&arch.entity().to_string()).unwrap().add_file(file)
            // create edges
            for dep in &arch.edges() {
                g.add_edge(map.get(dep).unwrap().index(), map.get(&arch.entity().to_string()).unwrap().index());
            }
        }

        // sort
        let order = g.topological_sort();
        println!("{:?}", order);
        println!("{:?}", map);

        // @TODO remove and properly include only-necessary in-order hdl files
        let vhdl_rtl_files = crate::core::fileset::collect_vhdl_files(&files, false);
        let vhdl_sim_files = crate::core::fileset::collect_vhdl_files(&files, true);
        


        // store data in blueprint TSV format
        let mut blueprint_data = String::new();

        // use command-line set filesets
        if let Some(fsets) = &self.filesets {
            for fset in fsets {
                let data = fset.collect_files(&files);
                for f in data {
                    blueprint_data += &format!("{}\t{}\t{}\n", fset.get_name(), std::path::PathBuf::from(f).file_stem().unwrap_or(&OsString::new()).to_str().unwrap(), f);
                }
            }
        }
        for f in vhdl_rtl_files {
            blueprint_data += &format!("VHDL-RTL\twork\t{}\n", f);
        }
        for f in vhdl_sim_files {
            blueprint_data += &format!("VHDL-SIM\twork\t{}\n", f);
        }
        // create a output build directorie(s) if they do not exist
        if std::path::PathBuf::from(build_dir).exists() == false {
            std::fs::create_dir_all(build_dir).expect("could not create build dir");
        }
        // create the blueprint file
        let blueprint_path = build_path.join("blueprint.tsv");
        let mut blueprint_file = std::fs::File::create(&blueprint_path).expect("could not create blueprint.tsv file");
        // write the data
        blueprint_file.write_all(blueprint_data.as_bytes()).expect("failed to write data to blueprint");
        
        // create environment variables to .env file
        let env_path = build_path.join(".env");
        let mut env_file = std::fs::File::create(&env_path).expect("could not create .env file");
        let contents = format!("ORBIT_TOP={}\nORBIT_BENCH={}\n", &self.top.as_ref().unwrap_or(&String::new()), &self.bench.as_ref().unwrap_or(&String::new()));
        // write the data
        env_file.write_all(contents.as_bytes()).expect("failed to write data to .env file");

        // create a blueprint file
        println!("info: Blueprint created at: {}", blueprint_path.display());
    }
}

impl FromCli for Plan {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Plan {
            top: cli.check_option(Optional::new("top").value("unit"))?,
            bench: cli.check_option(Optional::new("bench").value("tb"))?,
            plugin: cli.check_option(Optional::new("plugin"))?,
            build_dir: cli.check_option(Optional::new("build-dir").value("dir"))?,
            filesets: cli.check_option_all(Optional::new("fileset").value("key=glob"))?,
        });
        command
    }
}

const HELP: &str = "\
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