use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::core::pkgid::PkgPart;
use crate::core::vhdl::symbol::ResReference;
use crate::interface::cli::Cli;
use crate::util::anyerror::Fault;
use crate::util::environment::EnvVar;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::graphmap::GraphMap;
use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;
use crate::core::fileset::Fileset;
use crate::core::vhdl::token::Identifier;
use crate::core::plugin::Plugin;
use crate::util::environment;

#[derive(Debug, PartialEq)]
pub struct Plan {
    plugin: Option<String>,
    bench: Option<Identifier>,
    top: Option<Identifier>,
    clean: bool,
    list: bool,
    build_dir: Option<String>,
    filesets: Option<Vec<Fileset>>
}

impl Command for Plan {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // display plugin list and exit
        if self.list == true {
            println!("{}", Plugin::list_plugins(&c.get_plugins().values().into_iter().collect::<Vec<&Plugin>>()));
            return Ok(())
        }
        
        // check that user is in an IP directory
        c.goto_ip_path()?;

        // create the ip manifest
        let target_ip = IpManifest::from_path(c.get_ip_path().unwrap())?;

        // gather the catalog
        let catalog = Catalog::new()
            .store(c.get_store_path())
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(&&c.get_vendor_path())?;

        // set top-level environment variables (@TODO verify these are valid toplevels to be set!)
        if let Some(t) = &self.top {
            std::env::set_var(environment::ORBIT_TOP, t.to_string());
        }
        if let Some(b) = &self.bench {
            std::env::set_var(environment::ORBIT_BENCH, b.to_string());
        }
        // determine the build directory
        let b_dir = if let Some(dir) = &self.build_dir {
            dir
        } else {
            c.get_build_dir()
        };
        // find plugin filesets
        let plug_fset = if let Some(plug) = &self.plugin {
            match c.get_plugins().get(plug) {
                Some(p) => Some(p),
                None => return Err(AnyError(format!("plugin '{}' does not exist", plug)))?,
            }
        } else {
            None
        };

        self.run(target_ip, b_dir, plug_fset, catalog)
    }
}

use crate::core::vhdl::symbol;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct ArchitectureFile<'a> {
    architecture: symbol::Architecture,
    file: &'a IpFileNode<'a>,
}

impl<'a> ArchitectureFile<'a> {
    pub fn new(arch: symbol::Architecture, file: &'a IpFileNode<'a>) -> Self {
        Self { architecture: arch, file: file.to_owned() }
    }

    /// References the architecture struct.
    pub fn get_architecture(&self) -> &symbol::Architecture {
        &self.architecture
    }

    /// References the ip file node.
    pub fn get_file(&self) -> &'a IpFileNode<'a> {
        &self.file
    }
}

#[derive(Debug, PartialEq)]
pub struct GraphNode<'a> {
    sym: symbol::VHDLSymbol,
    files: Vec<&'a IpFileNode<'a>>, // must use a vector to retain file order in blueprint
}

impl<'a> GraphNode<'a> {
    fn new(sym: symbol::VHDLSymbol, file: &'a IpFileNode) -> Self {
        let mut set = Vec::with_capacity(1);
        set.push(file);
        Self {
            sym: sym,
            files: set,
        }
    }

    fn add_file(&mut self, ipf: &'a IpFileNode) {
        if self.files.contains(&ipf) == false {
            self.files.push(ipf);
        }
    }

    /// References the VHDL symbol
    fn get_symbol(&self) -> &symbol::VHDLSymbol {
        &self.sym
    }

    fn get_symbol_mut(&mut self) -> &mut symbol::VHDLSymbol {
        &mut self.sym
    }

    fn get_associated_files(&self) -> &Vec<&'a IpFileNode<'a>> {
        &self.files
    }
}

#[derive(Debug, PartialEq)]
pub struct IpFileNode<'a> {
    file: String,
    ip: &'a IpManifest
}

impl<'a> IpFileNode<'a> {
    pub fn new(file: String, ip: &'a IpManifest) -> Self {
        Self { file: file, ip: ip }
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    pub fn get_ip_manifest(&self) -> &IpManifest {
        &self.ip
    }

    /// References the library identifier from the ip's pkgid.
    pub fn get_library(&self) -> &PkgPart {
        &self.ip.get_pkgid().get_library().as_ref().unwrap()
    }
}

impl Plan {
    /// Constructs an entire list of dependencies required for the current design.
    /// 
    /// Errors if a dependency is not known in the user's catalog.
    fn construct_rough_build_list<'a>(target: &'a IpManifest, catalog: &'a Catalog) -> Result<Vec<&'a IpManifest>, Fault> {
        let mut result = vec![target];
        let mut processing = vec![target];

        while let Some(ip) = processing.pop() {
            let deps = ip.get_dependencies();
            for (pkgid, version) in deps.inner() {
                match catalog.inner().get(pkgid) {
                    Some(status) => {
                        // find this IP to read its dependencies
                        match status.get_install(version) {
                            Some(dep) => {
                                processing.push(dep)
                            },
                            None => panic!("ip is not installed"),
                        }
                        println!("found dependent ip: {}", pkgid);
                        result.push(ip);
                    },
                    None => return Err(AnyError(format!("unknown ip: {}", pkgid)))?
                }
            }
        }
        Ok(result)
    }

    /// Follow the Minimum Version Selection (MVS) algorithm to resolve dependencies.
    /// 
    /// MVS uses the "oldest allowed version" when selecting among packages with the same identifier.
    pub fn resolve_dependencies<'a>(target: &'a IpManifest, catalog: &'a Catalog) -> Result<Vec<&'a IpManifest>, Fault> {
        let rough_build = Self::construct_rough_build_list(target, catalog)?;
        Ok(rough_build)
    }

    /// Transforms the list of required `Ip` into list of all the available files.
    pub fn assemble_all_files<'a>(ips: Vec<&'a IpManifest>) -> Vec<IpFileNode> {
        let mut files = Vec::new();
        ips.iter().for_each(|ip| {
            crate::core::fileset::gather_current_files(&ip.get_root()).into_iter().for_each(|f| {
                files.push(IpFileNode { file: f, ip: ip });
            })
        });
        files
    }

    /// Builds a graph of design units. Used for planning.
    fn build_full_graph<'a>(files: &'a Vec<IpFileNode>) -> GraphMap<Identifier, GraphNode<'a>, ()> {
            let mut graph_map: GraphMap<Identifier, GraphNode, ()> = GraphMap::new();
    
            let mut archs: Vec<ArchitectureFile> = Vec::new();
            let mut bodies: Vec<symbol::PackageBody> = Vec::new();
            // read all files
            for source_file in files {
                if crate::core::fileset::is_vhdl(&source_file.file) == true {
                    let contents = std::fs::read_to_string(&source_file.file).unwrap();
                    let symbols = symbol::VHDLParser::read(&contents).into_symbols();
                    // add all entities to a graph and store architectures for later analysis
                    let mut iter = symbols.into_iter()
                        .filter_map(|f| {
                            match f {
                                symbol::VHDLSymbol::Entity(_) => Some(f),
                                symbol::VHDLSymbol::Package(_) => Some(f),
                                symbol::VHDLSymbol::Architecture(arch) => {
                                    archs.push(ArchitectureFile{ architecture: arch, file: source_file });
                                    None
                                }
                                // package bodies are usually in same design file as package
                                symbol::VHDLSymbol::PackageBody(pb) => {
                                    bodies.push(pb);
                                    None
                                }
                                _ => None,
                            }
                        });
                    while let Some(e) = iter.next() {
                        // add entities into the graph
                        graph_map.add_node(e.as_iden().unwrap().clone(), GraphNode::new(e, source_file));
                    }
                }
            }

            // go through all package bodies and update package dependencies
            let mut bodies = bodies.into_iter();
            while let Some(pb) = bodies.next() {
                // verify the package exists
                if let Some(p_node) = graph_map.get_node_by_key_mut(pb.get_owner()) {
                    // link to package owner by adding refs
                    p_node.as_ref_mut().get_symbol_mut().add_refs(&mut pb.take_refs());
                }
            }
    
            // go through all architectures and make the connections
            let mut archs = archs.into_iter();
            while let Some(af) = archs.next() {
                // link to the owner and add architecture's source file
                let entity_node = graph_map.get_node_by_key_mut(&af.architecture.entity()).unwrap();
                entity_node.as_ref_mut().add_file(af.file);
                // create edges
                for dep in af.architecture.edges() {
                    graph_map.add_edge_by_key(dep, af.architecture.entity(), ());
                }
                // add edges for reference calls
                for dep in af.architecture.get_refs() {
                    // note: verify the dependency exists (occurs within function)
                    graph_map.add_edge_by_key(dep.get_suffix(), af.architecture.entity(), ());
                }
            }

        // go through all nodes and make the connections
        let idens: Vec<Identifier> = graph_map.get_map().into_iter().map(|(k, _)| { k.clone() }).collect();
        for iden in idens {
            let references: Vec<ResReference> = graph_map.get_node_by_key(&iden).unwrap().as_ref().get_symbol().get_refs().into_iter().map(|rr| rr.clone() ).collect();
            for dep in &references {
                    // verify the dep exists
                    graph_map.add_edge_by_key(dep.get_suffix(), &iden, ());
            }
        }
        graph_map
    }

    /// Runs the backend logic for creating a blueprint file (planning a design).
    fn run(&self, target: IpManifest, build_dir: &str, plug: Option<&Plugin>, catalog: Catalog) -> Result<(), Fault> {
        let mut build_path = std::env::current_dir().unwrap();
        build_path.push(build_dir);
        
        // check if to clean the directory
        if self.clean == true && std::path::Path::exists(&build_path) == true {
            std::fs::remove_dir_all(&build_path)?;
        }

        // gather filesets
        let current_files = crate::core::fileset::gather_current_files(&std::env::current_dir().unwrap());
        let current_ip_nodes = current_files
            .into_iter()
            .map(|f| { IpFileNode { file: f, ip: &target }}).collect();
        // build full graph (all primary design units) and map storage
        let graph_map = Self::build_full_graph(&current_ip_nodes);

        let mut natural_top: Option<usize> = None;
        let mut bench = if let Some(t) = &self.bench {
            match graph_map.get_node_by_key(&t) {
                Some(node) => {
                    if let Some(e) = node.as_ref().get_symbol().as_entity() {
                        if e.is_testbench() == false {
                            return Err(PlanError::BadTestbench(t.clone()))?
                        }
                        Some(node.index())
                    } else {
                        return Err(PlanError::BadEntity(t.clone()))?
                    }
                },
                None => return Err(PlanError::UnknownEntity(t.clone()))?
            }
        } else if self.top.is_none() {
            // filter to display tops that have ports (not testbenches)
            match graph_map.find_root() {
                // only detected a single root
                Ok(n) => {
                    // verify the root is a testbench
                    if let Some(ent) = n.as_ref().get_symbol().as_entity() {
                        if ent.is_testbench() == true {
                            Some(n.index())
                        } else {
                            natural_top = Some(n.index());
                            None
                        }
                    } else {
                        None
                    }
                },
                Err(e) => {
                    match e.len() {
                        0 => None,
                        _ => return Err(PlanError::Ambiguous("testbenches".to_string(), e.into_iter().map(|f| { f.as_ref().get_symbol().as_iden().unwrap().clone() }).collect()))?,
                    }   
                }
            }
        } else {
            None // still could possibly be found by top level is top is some
        };

        // determine the top-level node index
        let top = if let Some(t) = &self.top {
            match graph_map.get_node_by_key(&t) {
                Some(node) => {
                    if let Some(e) = node.as_ref().get_symbol().as_entity() {
                        if e.is_testbench() == true {
                            return Err(PlanError::BadTop(t.clone()))?
                        }
                    } else {
                        return Err(PlanError::BadEntity(t.clone()))?
                    }
                    let n = node.index();
                    // try to detect top level testbench
                    if bench.is_none() {
                        // check if only 1 is a testbench
                        let benches: Vec<usize> =  graph_map.get_graph().successors(n)
                            .filter(|f| graph_map.get_node_by_index(*f).unwrap().as_ref().get_symbol().as_entity().unwrap().is_testbench() )
                            .collect();
                        bench = match benches.len() {
                            0 => None,
                            1 => Some(*benches.first().unwrap()),
                            _ => return Err(PlanError::Ambiguous("testbenches".to_string(), benches.into_iter().map(|f| { graph_map.get_key_by_index(f).unwrap().clone() }).collect()))?,
                        };
                    }
                    n
                },
                None => return Err(PlanError::UnknownEntity(t.clone()))?
            }
        } else {
            if let Some(nt) = natural_top {
                nt
            } else {
                Self::detect_top(&graph_map, bench)?
            }
        };
        // enable immutability
        let bench = bench;

        let top_name = graph_map.get_node_by_index(top).unwrap().as_ref().get_symbol().as_iden().unwrap().to_string();
        let bench_name = if let Some(n) = bench {
            graph_map.get_key_by_index(n).unwrap().to_string()
        } else {
            String::new()
        };

        let highest_point = match bench {
            Some(b) => b,
            None => top
        };

        let highest_iden = graph_map.get_key_by_index(highest_point).unwrap();

        // [!] build graph again but with entire set of all files available from all depdendencies
        let build_list = Self::resolve_dependencies(&target, &catalog)?;
        let files = Self::assemble_all_files(build_list);
        let graph_map = Self::build_full_graph(&files);
        // transfer identifier over the full graph
        let highest_point = graph_map.get_node_by_key(highest_iden).unwrap().index();

        std::env::set_var(environment::ORBIT_TOP, &top_name);
        std::env::set_var(environment::ORBIT_BENCH, &bench_name);

        // compute minimal topological ordering
        let min_order = graph_map.get_graph().minimal_topological_sort(highest_point);

        let mut file_order = Vec::new();
        for i in &min_order {
            // access the node key
            let ipfs = graph_map.get_node_by_index(*i).unwrap().as_ref().get_associated_files();
            // access the files associated with this key
            file_order.append(&mut ipfs.into_iter().map(|i| *i).collect());
        }

        // store data in blueprint TSV format
        let mut blueprint_data = String::new();

        // use command-line set filesets
        let current_files: Vec<String> = current_ip_nodes.into_iter().map(|f| f.file).collect();
        if let Some(fsets) = &self.filesets {
            for fset in fsets {
                let data = fset.collect_files(&current_files);
                for f in data {
                    blueprint_data += &format!("{}\t{}\t{}\n", fset.get_name(), std::path::PathBuf::from(f).file_stem().unwrap_or(&OsString::new()).to_str().unwrap(), f);
                }
            }
        }

        // collect data for the given plugin
        if let Some(p) = plug {
            let fsets = p.filesets();
            // define pattern matching settings
            let match_opts = glob::MatchOptions {
                case_sensitive: false,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            };
            // iterate through every collected file (from the current ip)
            for file in &current_files {
                // check against every defined fileset for the plugin
                for fset in fsets {
                    if fset.get_pattern().matches_with(&file, match_opts) == true {
                        // add to blueprint
                        blueprint_data += &fset.to_blueprint_string(&file);
                    }
                }
            }
        }

        // collect in-order hdl data
        for file in file_order {
            let lib = if current_files.contains(&file.file) == true {
                PkgPart::from_str("work").unwrap()
            } else {
                file.get_library().to_normal() // converts '-' to '_' for VHDL rules compatibility
            };
            if crate::core::fileset::is_rtl(&file.file) == true {
                blueprint_data += &format!("VHDL-RTL\t{}\t{}\n", lib, file.file);
            } else {
                blueprint_data += &format!("VHDL-SIM\t{}\t{}\n", lib, file.file);
            }
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
        let mut envs = environment::Environment::from_vec(vec![
            EnvVar::new().key(environment::ORBIT_TOP).value(&top_name), 
            EnvVar::new().key(environment::ORBIT_BENCH).value(&bench_name)
        ]);
        // conditionally set the plugin used to plan
        match plug {
            Some(p) => { envs.insert(EnvVar::new().key(environment::ORBIT_PLUGIN).value(&p.alias())); () },
            None => (),
        };
        crate::util::environment::save_environment(&envs, &build_path)?;

        // create a blueprint file
        println!("info: Blueprint created at: {}", blueprint_path.display());
        Ok(())
    }

    /// Given a `graph` and optionally a `bench`, detect the index corresponding
    /// to the top.
    /// 
    /// This function looks and checks if there is a single predecessor to the
    /// `bench` node.
    fn detect_top<'a>(graph_map: &GraphMap<Identifier, GraphNode<'a>, ()>, bench: Option<usize>) -> Result<usize, AnyError> {
        if let Some(b) = bench {
            let entities: Vec<(usize, &symbol::Entity)> = graph_map.get_graph().predecessors(b)
                .filter_map(|f| {
                    if let Some(e) = graph_map.get_node_by_index(f).unwrap().as_ref().get_symbol().as_entity() { 
                        Some((f, e)) } else { None }
                    })
                .collect();
            match entities.len() {
                0 => panic!("no entities are tested in the testbench"),
                1 => Ok(entities[0].0),
                _ => panic!("multiple entities are tested in testbench")
            }
        } else {
            todo!("find toplevel node that is not a bench")
        }
    }
}

impl FromCli for Plan {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Plan {
            top: cli.check_option(Optional::new("top").value("unit"))?,
            clean: cli.check_flag(Flag::new("clean"))?,
            list: cli.check_flag(Flag::new("list"))?,
            bench: cli.check_option(Optional::new("bench").value("tb"))?,
            plugin: cli.check_option(Optional::new("plugin"))?,
            build_dir: cli.check_option(Optional::new("build-dir").value("dir"))?,
            filesets: cli.check_option_all(Optional::new("fileset").value("key=glob"))?,
        });
        command
    }
}

#[derive(Debug)]
pub enum PlanError {
    BadTestbench(Identifier),
    BadTop(Identifier),
    BadEntity(Identifier),
    UnknownUnit(Identifier),
    UnknownEntity(Identifier),
    Ambiguous(String, Vec<Identifier>),
    Empty,
}

impl std::error::Error for PlanError {}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownEntity(id) => write!(f, "no entity named '{}' in the current ip", id),
            Self::Empty => write!(f, "no entities found"),
            Self::BadEntity(id) => write!(f, "primary design unit '{}' is not an entity", id),
            Self::BadTestbench(id) => write!(f, "entity '{}' is not a testbench and cannot be bench; use --top", id),
            Self::BadTop(id) => write!(f, "entity '{}' is a testbench and cannot be top; use --bench", id),
            Self::UnknownUnit(id) => write!(f, "no primary design unit named '{}' in the current ip", id),
            Self::Ambiguous(name, tbs) => write!(f, "multiple {} were found:\n {}", name, tbs.iter().fold(String::new(), |sum, x| {
                sum + &format!("\t{}\n", x)
            })),
        }
    }
}

const HELP: &str = "\
Generates a blueprint file.

Usage:
    orbit plan [options]              

Options:
    --top <unit>            override auto-detected toplevel entity
    --bench <tb>            override auto-detected toplevel testbench
    --plugin <alias>        collect filesets defined for a plugin
    --build-dir <dir>       set the output build directory
    --fileset <key=glob>... set an additional fileset
    --clean                 remove all files from the build directory
    --list                  view available plugins
    --all                   include all found HDL files

Use 'orbit help plan' to learn more about the command.
";