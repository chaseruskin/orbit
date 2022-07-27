use tempfile::tempdir;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::extgit;
use crate::core::ip::IpFileNode;
use crate::core::manifest::IpManifest;
use crate::core::lockfile::LockEntry;
use crate::core::template;
use crate::core::variable::VariableTable;
use crate::core::version::AnyVersion;
use crate::core::vhdl::subunit::SubUnit;
use crate::core::vhdl::symbol::CompoundIdentifier;
use crate::interface::cli::Cli;
use crate::util::anyerror::Fault;
use crate::util::environment::EnvVar;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::graphmap::GraphMap;
use std::collections::HashMap;
use std::io::Write;
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
    filesets: Option<Vec<Fileset>>,
    disable_ssh: bool,
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
        let mut catalog = Catalog::new()
            .store(c.get_store_path())
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(c.get_vendors())?;

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if target_ip.can_use_lock() == true && c.force == false {
            // fill in the catalog with missing modules according the lock file if available
            for entry in target_ip.into_lockfile()?.inner() {
                // skip the current project's ip entry
                if entry.get_name() == target_ip.get_pkgid() { continue }
                let ver = AnyVersion::Specific(entry.get_version().to_partial_version());
                // try to use the lock file to fill in missing pieces
                match catalog.inner().get(entry.get_name()) {
                    Some(status) => {
                        // find this IP to read its dependencies
                        match status.get(&ver, true) {
                            // no action required
                            Some(_) => (),
                            // install
                            None => Plan::install_from_lock_entry(&entry, &ver, &catalog, self.disable_ssh)?,
                        }
                    }
                    // install
                    None => Plan::install_from_lock_entry(&entry, &ver, &catalog, self.disable_ssh)?,
                }
            }
            // recollect the installations to update the catalog
            catalog = catalog.installations(c.get_cache_path())?;
        }

        // determine the build directory (command-line arg overrides configuration setting)
        let b_dir = match &self.build_dir {
            Some(dir) => dir,
            None => c.get_build_dir(),
        };

        // locate the plugin
        let plugin = match &self.plugin {
            // verify the plugin alias matches
            Some(alias) => match c.get_plugins().get(alias) {
                Some(p) => Some(p),
                None => return Err(AnyError(format!("plugin '{}' does not exist", alias)))?,
            },
            None => None,
        };

        self.run(target_ip, b_dir, plugin, catalog)
    }
}

use crate::core::vhdl::symbol;
use crate::util::anyerror::AnyError;

use super::install;

#[derive(Debug, PartialEq)]
pub struct SubUnitNode<'a> {
    sub: SubUnit,
    file: &'a IpFileNode<'a>,
}

impl<'a> SubUnitNode<'a> {
    pub fn new(unit: SubUnit, file: &'a IpFileNode<'a>) -> Self {
        Self { sub: unit, file: file }
    }

    /// References the architecture struct.
    pub fn get_sub(&self) -> &SubUnit {
        &self.sub
    }

    /// References the ip file node.
    pub fn get_file(&self) -> &'a IpFileNode<'a> {
        &self.file
    }
}

#[derive(Debug, PartialEq)]
pub struct HdlNode<'a> {
    sym: symbol::VHDLSymbol,
    files: Vec<&'a IpFileNode<'a>>, // must use a vector to retain file order in blueprint
}

impl<'a> HdlNode<'a> {
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

impl Plan {
    /// Clones the ip entry's repository to a temporary directory and then installs the appropriate version `ver`.
    fn install_from_lock_entry(entry: &LockEntry, ver: &AnyVersion, catalog: &Catalog, disable_ssh: bool) -> Result<(), Fault> {
        let temp = tempdir()?;
        // try to use the source
        let from = if let Some(source) = entry.get_source() {
            let temp = temp.as_ref().to_path_buf();
            println!("info: fetching {} repository ...", entry.get_name());
            extgit::ExtGit::new(None)
                .clone(source, &temp, disable_ssh)?;
            temp
        // try to find an install path
        } else {
            install::fetch_install_path(entry.get_name(), &catalog, disable_ssh, &temp)?
        };
        let ip = install::Install::install(&from, &ver, catalog.get_cache_path(), true, catalog.get_store())?;

        // verify the checksums align
        match &ip.read_checksum_proof().unwrap() == entry.get_sum().unwrap() {
            true => Ok(()),
            false => {
                // delete the entry from the cache slot
                ip.remove()?;
                Err(AnyError(format!("failed to install ip '{}' from lockfile due to differing checksums\n\ncomputed: {}\nexpected: {}", entry.get_name(), ip.read_checksum_proof().unwrap(), entry.get_sum().unwrap())))?
            }
        } 
    }

    /// Builds a graph of design units. Used for planning.
    fn build_full_graph<'a>(files: &'a Vec<IpFileNode>) -> GraphMap<CompoundIdentifier, HdlNode<'a>, ()> {
            let mut graph_map: GraphMap<CompoundIdentifier, HdlNode, ()> = GraphMap::new();
    
            let mut sub_nodes: Vec<(Identifier, SubUnitNode)> = Vec::new();
            let mut bodies: Vec<(Identifier, symbol::PackageBody)> = Vec::new();
            // store the (suffix, prefix) for all entities
            let mut component_pairs: HashMap<Identifier, Identifier> = HashMap::new();
            // read all files
            for source_file in files {
                if crate::core::fileset::is_vhdl(&source_file.get_file()) == true {
                    let contents = std::fs::read_to_string(&source_file.get_file()).unwrap();
                    let symbols = symbol::VHDLParser::read(&contents).into_symbols();

                    let lib = source_file.get_library();

                    // add all entities to a graph and store architectures for later analysis
                    let mut iter = symbols.into_iter()
                        .filter_map(|f| {
                            match f {
                                symbol::VHDLSymbol::Entity(_) => {
                                    component_pairs.insert(f.as_entity().unwrap().get_name().clone(), lib.clone());
                                    Some(f)
                                },
                                symbol::VHDLSymbol::Package(_) => Some(f),
                                symbol::VHDLSymbol::Context(_) => Some(f),
                                symbol::VHDLSymbol::Architecture(arch) => {
                                    sub_nodes.push((lib.clone(), SubUnitNode{ sub: SubUnit::from_arch(arch), file: source_file }));
                                    None
                                }
                                symbol::VHDLSymbol::Configuration(cfg) => {
                                    sub_nodes.push((lib.clone(), SubUnitNode { sub: SubUnit::from_config(cfg), file: source_file }));
                                    None
                                }
                                // package bodies are usually in same design file as package
                                symbol::VHDLSymbol::PackageBody(pb) => {
                                    bodies.push((lib.clone(), pb));
                                    None
                                }
                            }
                        });
                    while let Some(e) = iter.next() {
                        // add primary design units into the graph
                        graph_map.add_node(
                            CompoundIdentifier::new(
                                Identifier::from(lib.clone()), 
                                e.as_iden().unwrap().clone()), 
                            HdlNode::new(e, source_file)
                            );
                    }
                }
            }

            // go through all package bodies and update package dependencies
            let mut bodies = bodies.into_iter();
            while let Some((lib, pb)) = bodies.next() {
                // verify the package exists
                if let Some(p_node) = graph_map.get_node_by_key_mut(&CompoundIdentifier::new(lib, pb.get_owner().clone())) {
                    // link to package owner by adding refs
                    p_node.as_ref_mut().get_symbol_mut().add_refs(&mut pb.take_refs());
                }
            }
    
            // go through all architectures and make the connections
            let mut sub_nodes_iter = sub_nodes.into_iter();
            while let Some((lib, node)) = sub_nodes_iter.next() {

                let node_name = CompoundIdentifier::new(lib, node.get_sub().get_entity().clone());
        
                // link to the owner and add architecture's source file
                let entity_node = graph_map.get_node_by_key_mut(&node_name).unwrap();
                entity_node.as_ref_mut().add_file(node.file);
                // create edges
                for dep in node.get_sub().get_edges() {
                    // need to locate the key with a suffix matching `dep` if it was a component instantiation
                    if dep.get_prefix().is_none() {
                        if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                            graph_map.add_edge_by_key(&CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()), &node_name, ());
                        }
                    } else {
                        graph_map.add_edge_by_key(dep, &node_name, ());
                    };
                    
                }
                // add edges for reference calls
                for dep in node.get_sub().get_refs() {
                    // note: verify the dependency exists (occurs within function)
                    graph_map.add_edge_by_key(dep, &node_name, ());
                }
            }

        // go through all nodes and make the connections
        let idens: Vec<CompoundIdentifier> = graph_map.get_map().into_iter().map(|(k, _)| { k.clone() }).collect();
        for iden in idens {
            let references: Vec<CompoundIdentifier> = graph_map.get_node_by_key(&iden).unwrap().as_ref().get_symbol().get_refs().into_iter().map(|rr| rr.clone() ).collect();
            for dep in &references {
                    // verify the dep exists
                    graph_map.add_edge_by_key(dep, &iden, ());
            }
        }
        graph_map
    }

    /// Performs the backend logic for creating a blueprint file (planning a design).
    fn run(&self, target: IpManifest, build_dir: &str, plug: Option<&Plugin>, catalog: Catalog) -> Result<(), Fault> {
        // create the build path to know where to begin storing files
        let mut build_path = std::env::current_dir().unwrap();
        build_path.push(build_dir);
        
        // check if to clean the directory
        if self.clean == true && std::path::Path::exists(&build_path) == true {
            std::fs::remove_dir_all(&build_path)?;
        }

        // gather filesets
        let current_files = crate::util::filesystem::gather_current_files(&std::env::current_dir().unwrap());
        let current_ip_nodes = current_files
            .into_iter()
            .map(|f| { IpFileNode::new(f, &target, Identifier::new_working()) }).collect();
        // build shallow graph (all primary design units)
        let current_graph = Self::build_full_graph(&current_ip_nodes);

        let working_lib = Identifier::new_working();

        let mut natural_top: Option<usize> = None;
        let mut bench = if let Some(t) = &self.bench {
            match current_graph.get_node_by_key(&CompoundIdentifier::new(working_lib.clone(), t.clone())) {
                // verify the unit is an entity that is a testbench
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
            match current_graph.find_root() {
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
                        _ => return Err(PlanError::Ambiguous("roots".to_string(), e.into_iter().map(|f| { f.as_ref().get_symbol().as_iden().unwrap().clone() }).collect()))?,
                    }   
                }
            }
        } else {
            None // still could possibly be found by top level if top is some
        };

        // determine the top-level node index
        let top = if let Some(t) = &self.top {
            match current_graph.get_node_by_key(&CompoundIdentifier::new(working_lib.clone(), t.clone())) {
                Some(node) => {
                    // verify the unit is an entity that is not a testbench
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
                        let benches: Vec<usize> =  current_graph.get_graph().successors(n)
                            .filter(|f| current_graph.get_node_by_index(*f).unwrap().as_ref().get_symbol().as_entity().unwrap().is_testbench() )
                            .collect();
                        bench = match benches.len() {
                            0 => None,
                            1 => Some(*benches.first().unwrap()),
                            _ => return Err(PlanError::Ambiguous("testbenches".to_string(), benches.into_iter().map(|f| { current_graph.get_key_by_index(f).unwrap().get_suffix().clone() }).collect()))?,
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
                Self::detect_top(&current_graph, bench)?
            }
        };
        // set immutability
        let bench = bench;

        let top_name = current_graph.get_node_by_index(top).unwrap().as_ref().get_symbol().as_iden().unwrap().to_string();
        let bench_name = if let Some(n) = bench {
            current_graph.get_key_by_index(n).unwrap().get_suffix().to_string()
        } else {
            String::new()
        };

        // determine which point is the upmost root 
        let highest_point = match bench {
            Some(b) => b,
            None => top
        };

        let highest_iden = current_graph.get_key_by_index(highest_point).unwrap();
        // build entire ip graph and resolve with dynamic symbol transformation
        let ip_graph = crate::core::ip::compute_final_ip_graph(&target, &catalog)?;
        
        // [!] write the lock file
        {
            // create build list
            let mut build_list: Vec<&IpManifest> = ip_graph.get_map()
                .iter()
                .map(|p| { p.1.as_ref().as_original_ip() })
                .collect();
            target.write_lock(&mut build_list)?;
        }

        let files = crate::core::ip::build_ip_file_list(&ip_graph);
        let graph_map = Self::build_full_graph(&files);

        // transfer identifier over to the full graph
        let highest_point = graph_map.get_node_by_key(highest_iden).unwrap().index();

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

        let current_files: Vec<String> = current_ip_nodes.into_iter()
            .map(|f| f.get_file().to_owned())
            .collect();

        {
            let mut vtable = VariableTable::new();
            vtable.add("orbit.bench", &bench_name);
            vtable.add("orbit.top", &top_name);
    
            // use command-line set filesets
            if let Some(fsets) = &self.filesets {
                for fset in fsets {
                    // perform variable substitution
                    let fset = Fileset::new()
                        .name(fset.get_name())
                        .pattern(&template::substitute(fset.get_pattern().to_string(), &vtable))?;
                    // match files
                    fset.collect_files(&current_files).into_iter().for_each(|f| {
                        blueprint_data += &fset.to_blueprint_string(f);
                    });
                }
            }
    
            // collect data for the given plugin
            if let Some(p) = plug {
                let fsets = p.filesets();
                // check against every defined fileset for the plugin
                for fset in fsets {
                    // perform variable substitution
                    let fset = Fileset::new()
                        .name(fset.get_name())
                        .pattern(&template::substitute(fset.get_pattern().to_string(), &vtable))?;
                    // match files
                    fset.collect_files(&current_files).into_iter().for_each(|f| {
                        blueprint_data += &fset.to_blueprint_string(&f);
                    });
                }
            }
        }

        // collect in-order HDL file list
        for file in file_order {
            let lib = file.get_library();
            if crate::core::fileset::is_rtl(&file.get_file()) == true {
                blueprint_data += &format!("VHDL-RTL\t{}\t{}\n", lib, file.get_file());
            } else {
                blueprint_data += &format!("VHDL-SIM\t{}\t{}\n", lib, file.get_file());
            }
        }

        // create a output build directorie(s) if they do not exist
        if std::path::PathBuf::from(build_dir).exists() == false {
            std::fs::create_dir_all(build_dir).expect("could not create build dir");
        }

        // [!] create the blueprint file
        let blueprint_path = build_path.join(BLUEPRINT_FILE);
        let mut blueprint_file = std::fs::File::create(&blueprint_path).expect("could not create blueprint file");
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
    fn detect_top<'a>(graph_map: &GraphMap<CompoundIdentifier, HdlNode<'a>, ()>, bench: Option<usize>) -> Result<usize, Fault> {
        if let Some(b) = bench {
            let entities: Vec<(usize, &symbol::Entity)> = graph_map.get_graph().predecessors(b)
                .filter_map(|f| {
                    if let Some(e) = graph_map.get_node_by_index(f).unwrap().as_ref().get_symbol().as_entity() { 
                        Some((f, e)) } else { None }
                    })
                .collect();
            match entities.len() {
                0 => Err(AnyError(format!("no entities are tested in the testbench")))?,
                1 => Ok(entities[0].0),
                _ => Err(PlanError::Ambiguous("entities instantiated in the testbench".to_string(), entities.into_iter().map(|f| { graph_map.get_key_by_index(f.0).unwrap().get_suffix().clone() }).collect()))?
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
            disable_ssh: cli.check_flag(Flag::new("disable-ssh"))?,
        });
        command
    }
}

pub const BLUEPRINT_FILE: &str = "blueprint.tsv";

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
    --disable-ssh           convert SSH repositories to HTTPS for dependencies
    --force                 skip reading from the lock file

Use 'orbit help plan' to learn more about the command.
";