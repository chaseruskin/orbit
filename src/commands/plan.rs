use colored::Colorize;

use clif::cmd::{Command, FromCli};

use crate::commands::download::Download;
use crate::core::context::Context;
use crate::core::fileset::Fileset;
use crate::core::iparchive::IpArchive;
use crate::core::lang::vhdl::subunit::SubUnit;
use crate::core::lang::vhdl::symbol::CompoundIdentifier;
use crate::core::lang::vhdl::symbol::{Entity, PackageBody, VHDLParser, VHDLSymbol};
use crate::core::lang::vhdl::token::Identifier;
use crate::core::plugin::Plugin;
use crate::core::plugin::PluginError;
use crate::core::variable;
use crate::core::variable::VariableTable;
use crate::core::version::AnyVersion;
use crate::util::anyerror::Fault;
use crate::util::environment;
use crate::util::environment::EnvVar;
use crate::util::environment::Environment;
use crate::util::filesystem;
use crate::util::graphmap::GraphMap;
use crate::OrbitResult;
use clif::arg::{Flag, Optional};
use clif::Cli;
use clif::Error as CliError;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::commands::install::Install;
use crate::core::algo;
use crate::core::algo::IpFileNode;
use crate::core::algo::IpNode;
use crate::core::catalog::Catalog;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::lockfile::LockEntry;
use crate::core::lockfile::LockFile;

use crate::util::graphmap::Node;

pub const BLUEPRINT_FILE: &str = "blueprint.tsv";
pub const BLUEPRINT_DELIMITER: &str = "\t";

#[derive(Debug, PartialEq)]
pub struct Plan {
    plugin: Option<String>,
    bench: Option<Identifier>,
    top: Option<Identifier>,
    clean: bool,
    list: bool,
    all: bool,
    build_dir: Option<String>,
    filesets: Option<Vec<Fileset>>,
    only_lock: bool,
    force: bool,
}

impl FromCli for Plan {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Plan {
            force: cli.check_flag(Flag::new("force"))?,
            only_lock: cli.check_flag(Flag::new("lock-only"))?,
            all: cli.check_flag(Flag::new("all"))?,
            clean: cli.check_flag(Flag::new("clean"))?,
            list: cli.check_flag(Flag::new("list"))?,
            top: cli.check_option(Optional::new("top").value("unit"))?,
            bench: cli.check_option(Optional::new("bench").value("tb"))?,
            plugin: cli.check_option(Optional::new("plugin"))?,
            build_dir: cli.check_option(Optional::new("build-dir").value("dir"))?,
            filesets: cli.check_option_all(Optional::new("fileset").value("key=glob"))?,
        });
        command
    }
}

impl Command<Context> for Plan {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // locate the plugin
        let plugin = match &self.plugin {
            // verify the plugin alias matches
            Some(alias) => match c.get_config().get_plugins().get(alias.as_str()) {
                Some(&p) => Some(p),
                None => return Err(PluginError::Missing(alias.to_string()))?,
            },
            None => None,
        };

        // display plugin list and exit
        if self.list == true {
            match plugin {
                // display entire contents about the particular plugin
                Some(plg) => println!("{}", plg),
                // display quick overview of all plugins
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

        // check that user is in an IP directory
        c.goto_ip_path()?;

        // create the ip manifest
        let target = Ip::load(c.get_ip_path().unwrap().clone())?;

        // gather the catalog
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        // @todo: recreate the ip graph from the lockfile, then read each installation
        // see Install::install_from_lock_file

        // this code is only ran if the lock file matches the manifest and we aren't force to recompute
        if target.can_use_lock() == true && self.force == false {
            let le: LockEntry = LockEntry::from((&target, true));
            let lf = target.get_lock();

            let env = Environment::new()
                // read config.toml for setting any env variables
                .from_config(c.get_config())?;
            let vtable = VariableTable::new().load_environment(&env)?;

            download_missing_deps(vtable, &lf, &le, &catalog, &c.get_config().get_protocols())?;
            // recollect the downloaded items to update the catalog for installations
            catalog = catalog.downloads(c.get_downloads_path())?;

            install_missing_deps(&lf, &le, &catalog)?;
            // recollect the installations to update the catalog for dependency graphing
            catalog = catalog.installations(c.get_cache_path())?;
        }

        // determine the build directory (command-line arg overrides configuration setting)
        let b_dir = match &self.build_dir {
            Some(dir) => dir,
            None => c.get_build_dir(),
        };

        self.run(target, b_dir, plugin, catalog)
    }
}

pub fn download_missing_deps(
    vtable: VariableTable,
    lf: &LockFile,
    le: &LockEntry,
    catalog: &Catalog,
    protocols: &ProtocolMap,
) -> Result<(), Fault> {
    let mut vtable = vtable;
    // fetch all non-downloaded packages
    for entry in lf.inner() {
        // skip the current project's IP entry or any IP already in the downloads/
        if entry.matches_target(le) == true || catalog.is_downloaded_slot(&entry.to_download_slot_key()) == true {
            continue;
        }

        let ver = AnyVersion::Specific(entry.get_version().to_partial_version());

        let mut require_download = false;

        match catalog.inner().get(entry.get_name()) {
            Some(status) => {
                match status.get_install(&ver) {
                    Some(dep) => {
                        // verify the checksum
                        if Install::is_checksum_good(&dep.get_root()) == false {
                            println!(
                                "info: Redownloading IP {} due to bad checksum ...",
                                dep.get_man().get_ip().into_ip_spec()
                            );
                            require_download = true;
                        }
                    }
                    None => {
                        match status.get_download(&ver) {
                            // already exists in the downloads
                            Some(_) => (),
                            // does not exist in the downloads
                            None => {
                                require_download = true;
                            }
                        }
                    }
                }
            }
            // does not exist at all in the catalog
            None => {
                require_download = true;
            }
        }
        // check if the slot is not already filled before trying to download
        if require_download == true {
            match entry.get_source() {
                Some(src) => {
                    // fetch from the internet
                    Download::download(
                        &mut vtable,
                        &entry.to_ip_spec(),
                        src,
                        None,
                        catalog.get_downloads_path(),
                        &protocols,
                        false,
                        true,
                    )?;
                }
                None => {
                    return Err(AnyError(format!(
                        "unable to fetch IP {} from the internet due to missing source",
                        entry.to_ip_spec()
                    )))?;
                }
            }
        }
    }
    Ok(())
}

pub fn install_missing_deps(lf: &LockFile, le: &LockEntry, catalog: &Catalog) -> Result<(), Fault> {
    // fill in the catalog with missing modules according the lock file if available
    for entry in lf.inner() {
        // skip the current project's IP entry
        if entry.matches_target(&le) {
            continue;
        }

        let ver = AnyVersion::Specific(entry.get_version().to_partial_version());

        // try to use the lock file to fill in missing pieces
        match catalog.inner().get(entry.get_name()) {
            Some(status) => {
                // find this IP to read its dependencies
                match status.get_install(&ver) {
                    // no action required (already installed)
                    Some(dep) => {
                        // verify the checksum in case we need to re-install from downloads
                        if Install::is_checksum_good(&dep.get_root()) == false {
                            match status.get_download(&ver) {
                                Some(dep) => {
                                    println!(
                                        "info: Reinstalling IP {} due to bad checksum ...",
                                        dep.get_man().get_ip().into_ip_spec()
                                    );
                                    // perform extra work if the Ip is virtual (from downloads)
                                    install_ip_from_downloads(&dep, &catalog, true)?
                                }
                                None => {
                                    // failed to get the install from the queue
                                    panic!(
                                        "entry is not queued for installation (missing download)"
                                    )
                                }
                            }
                        }
                    }
                    // install
                    None => {
                        // check the queue for installation
                        match status.get_download(&ver) {
                            Some(dep) => {
                                // perform extra work if the Ip is virtual (from downloads)
                                install_ip_from_downloads(&dep, &catalog, false)?
                            }
                            None => {
                                panic!("entry is not queued for installation")
                            }
                        }
                    }
                }
            }
            None => {
                panic!("entry is not queued for installation (unknown ip)")
            }
        }
    }
    Ok(())
}

fn install_ip_from_downloads(dep: &Ip, catalog: &Catalog, force: bool) -> Result<(), Fault> {
    // perform extra work if the Ip is virtual (from downloads)
    if let Some(bytes) = dep.get_mapping().as_bytes() {
        // place the dependency into a temporary directory
        let dir = tempfile::tempdir()?.into_path();
        if let Err(e) = IpArchive::extract(&bytes, &dir) {
            fs::remove_dir_all(dir)?;
            return Err(e);
        }
        // load the IP
        let unzipped_dep = match Ip::load(dir.clone()) {
            Ok(x) => x,
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        };
        // install from the unzipp ip
        match Install::install(&unzipped_dep, catalog.get_cache_path(), force) {
            Ok(_) => {}
            Err(e) => {
                fs::remove_dir_all(dir)?;
                return Err(e);
            }
        }
        fs::remove_dir_all(unzipped_dep.get_root())?;
    } else {
        panic!("trying to download from a physical path")
    }
    Ok(())
}

use crate::core::fileset;
use crate::util::anyerror::AnyError;

use super::download::ProtocolMap;

use crate::core::lang::node::HdlNode;
use crate::core::lang::node::SubUnitNode;

impl Plan {
    /// Builds a graph of design units. Used for planning.
    fn build_full_graph<'a>(
        files: &'a Vec<IpFileNode>,
    ) -> GraphMap<CompoundIdentifier, HdlNode<'a>, ()> {
        let mut graph_map: GraphMap<CompoundIdentifier, HdlNode, ()> = GraphMap::new();

        let mut sub_nodes: Vec<(Identifier, SubUnitNode)> = Vec::new();
        let mut bodies: Vec<(Identifier, PackageBody)> = Vec::new();
        // store the (suffix, prefix) for all entities
        let mut component_pairs: HashMap<Identifier, Identifier> = HashMap::new();
        // read all files
        for source_file in files {
            if fileset::is_vhdl(&source_file.get_file()) == true {
                let contents = fs::read_to_string(&source_file.get_file()).unwrap();
                let symbols = VHDLParser::read(&contents).into_symbols();

                let lib = source_file.get_library();
                // println!("{} {}", source_file.get_file(), source_file.get_library());

                // add all entities to a graph and store architectures for later analysis
                let mut iter = symbols.into_iter().filter_map(|f| {
                    match f {
                        VHDLSymbol::Entity(_) => {
                            component_pairs
                                .insert(f.as_entity().unwrap().get_name().clone(), lib.clone());
                            Some(f)
                        }
                        VHDLSymbol::Package(_) => Some(f),
                        VHDLSymbol::Context(_) => Some(f),
                        VHDLSymbol::Architecture(arch) => {
                            sub_nodes.push((
                                lib.clone(),
                                SubUnitNode::new(SubUnit::from_arch(arch), source_file),
                            ));
                            None
                        }
                        VHDLSymbol::Configuration(cfg) => {
                            sub_nodes.push((
                                lib.clone(),
                                SubUnitNode::new(SubUnit::from_config(cfg), source_file),
                            ));
                            None
                        }
                        // package bodies are usually in same design file as package
                        VHDLSymbol::PackageBody(pb) => {
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
                            e.as_iden().unwrap().clone(),
                        ),
                        HdlNode::new(e, source_file),
                    );
                }
            }
        }

        // go through all package bodies and update package dependencies
        let mut bodies = bodies.into_iter();
        while let Some((lib, pb)) = bodies.next() {
            // verify the package exists
            if let Some(p_node) =
                graph_map.get_node_by_key_mut(&CompoundIdentifier::new(lib, pb.get_owner().clone()))
            {
                // link to package owner by adding refs
                p_node
                    .as_ref_mut()
                    .get_symbol_mut()
                    .add_refs(&mut pb.take_refs());
            }
        }

        // go through all architectures and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some((lib, node)) = sub_nodes_iter.next() {
            let node_name = CompoundIdentifier::new(lib, node.get_sub().get_entity().clone());

            // link to the owner and add architecture's source file
            let entity_node = match graph_map.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                // @todo: issue error because the entity (owner) is not declared
                None => continue,
            };
            entity_node.as_ref_mut().add_file(node.get_file());
            // create edges
            for dep in node.get_sub().get_edges() {
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        graph_map.add_edge_by_key(
                            &CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()),
                            &node_name,
                            (),
                        );
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
        let idens: Vec<CompoundIdentifier> = graph_map
            .get_map()
            .into_iter()
            .map(|(k, _)| k.clone())
            .collect();
        for iden in idens {
            let references: Vec<CompoundIdentifier> = graph_map
                .get_node_by_key(&iden)
                .unwrap()
                .as_ref()
                .get_symbol()
                .get_refs()
                .into_iter()
                .map(|rr| rr.clone())
                .collect();

            for dep in &references {
                let working = Identifier::Basic("work".to_string());
                // re-route the library prefix to the current unit's library
                let dep_adjusted = CompoundIdentifier::new(iden.get_prefix().unwrap_or(&working).clone(), dep.get_suffix().clone());
                // if the dep is using "work", match it with the identifier's library
                let dep_adjusted = if let Some(lib) = dep.get_prefix() {
                    match lib == &working {
                        true => &dep_adjusted,
                        false => dep,
                    }
                } else {
                    dep
                };
                // println!("{} {} ... {}", iden, dep, dep_adjusted);
                // verify the dep exists
                let _stat = graph_map.add_edge_by_key(dep_adjusted, &iden, ());
                // println!("{:?}", stat);
            }
        }
        graph_map
    }

    /// Writes the lockfile according to the constructed `ip_graph`. Only writes if the lockfile is
    /// out of date or `force` is `true`.
    pub fn write_lockfile(
        target: &Ip,
        ip_graph: &GraphMap<IpSpec, IpNode, ()>,
        force: bool,
    ) -> Result<(), Fault> {
        // only modify the lockfile if it is out-of-date
        if target.can_use_lock() == false || force == true {
            // create build list
            let mut build_list: Vec<&Ip> = ip_graph
                .get_map()
                .iter()
                .map(|p| p.1.as_ref().as_original_ip())
                .collect();
            let lock = LockFile::from_build_list(&mut build_list, target);
            lock.save_to_disk(target.get_root())?;
            println!("info: Updated lockfile");
        } else {
            println!("info: Lockfile is already up to date");
        }
        Ok(())
    }

    /// Maps the local index to the global index between two different maps.
    ///
    /// Assumes `local` is a subset of `global`.
    pub fn local_to_global<'a>(
        local_index: usize,
        global: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
    ) -> &'a Node<HdlNode<'a>> {
        global
            .get_node_by_key(local.get_key_by_index(local_index).unwrap())
            .unwrap()
    }

    fn detect_bench(
        &self,
        _graph: &GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
        working_lib: &Identifier,
    ) -> Result<(Option<usize>, Option<usize>), PlanError> {
        Ok(if let Some(t) = &self.bench {
            match local.get_node_by_key(&&CompoundIdentifier::new(working_lib.clone(), t.clone())) {
                // verify the unit is an entity that is a testbench
                Some(node) => {
                    if let Some(e) = node.as_ref().get_symbol().as_entity() {
                        if e.is_testbench() == false {
                            return Err(PlanError::BadTestbench(t.clone()))?;
                        }
                        // return the id from the local graph
                        (None, Some(node.index()))
                    } else {
                        return Err(PlanError::BadEntity(t.clone()))?;
                    }
                }
                None => return Err(PlanError::UnknownEntity(t.clone()))?,
            }
        // try to find the naturally occurring top-level if user did not provide --bench and did not provide --top
        } else if self.top.is_none() {
            match local.find_root() {
                // only detected a single root
                Ok(n) => {
                    let n = local
                        .get_node_by_key(local.get_key_by_index(n.index()).unwrap())
                        .unwrap();
                    // verify the root is a testbench
                    if let Some(ent) = n.as_ref().get_symbol().as_entity() {
                        if ent.is_testbench() == true {
                            (None, Some(n.index()))
                        // otherwise we found the toplevel node that is not a testbench "natural top"
                        } else {
                            // return the local index
                            (Some(n.index()), None)
                        }
                    } else {
                        (None, None)
                    }
                }
                Err(e) => match e.len() {
                    0 => (None, None),
                    _ => {
                        return Err(PlanError::Ambiguous(
                            "roots".to_string(),
                            e.into_iter()
                                .map(|f| f.as_ref().get_symbol().as_iden().unwrap().clone())
                                .collect(),
                        ))?
                    }
                },
            }
        } else {
            // still could possibly be found by top level if top is some
            (None, None)
        })
    }

    /// Given a `graph` and optionally a `bench`, detect the index corresponding
    /// to the top.
    ///
    /// This function looks and checks if there is a single predecessor to the
    /// `bench` node.
    fn detect_top(
        &self,
        _graph: &GraphMap<CompoundIdentifier, HdlNode, ()>,
        local: &GraphMap<&CompoundIdentifier, &HdlNode, &()>,
        working_lib: &Identifier,
        natural_top: Option<usize>,
        mut bench: Option<usize>,
    ) -> Result<(Option<usize>, Option<usize>), PlanError> {
        // determine the top-level node index
        let top: Option<usize> = if let Some(t) = &self.top {
            match local.get_node_by_key(&&CompoundIdentifier::new(working_lib.clone(), t.clone())) {
                Some(node) => {
                    // verify the unit is an entity that is not a testbench
                    if let Some(e) = node.as_ref().get_symbol().as_entity() {
                        if e.is_testbench() == true {
                            return Err(PlanError::BadTop(t.clone()))?;
                        }
                    } else {
                        return Err(PlanError::BadEntity(t.clone()))?;
                    }
                    let n: usize = node.index();
                    // try to detect top level testbench
                    if bench.is_none() {
                        // check if only 1 is a testbench
                        let benches: Vec<usize> = local
                            .get_graph()
                            .successors(n)
                            .filter(|f| {
                                local
                                    .get_node_by_index(*f)
                                    .unwrap()
                                    .as_ref()
                                    .get_symbol()
                                    .as_entity()
                                    .unwrap()
                                    .is_testbench()
                            })
                            .collect();
                        // detect the testbench
                        bench = match benches.len() {
                            0 => None,
                            1 => Some(*benches.first().unwrap()),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    "testbenches".to_string(),
                                    benches
                                        .into_iter()
                                        .map(|f| {
                                            local.get_key_by_index(f).unwrap().get_suffix().clone()
                                        })
                                        .collect(),
                                ))?
                            }
                        };
                    }
                    // return the index from the local graph
                    Some(n)
                }
                None => return Err(PlanError::UnknownEntity(t.clone()))?,
            }
        } else {
            match natural_top {
                Some(nt) => Some(nt),
                None => {
                    if let Some(b) = bench {
                        let entities: Vec<(usize, &Entity)> = local
                            .get_graph()
                            .predecessors(b)
                            .filter_map(|f| {
                                if let Some(e) = local
                                    .get_node_by_index(f)
                                    .unwrap()
                                    .as_ref()
                                    .get_symbol()
                                    .as_entity()
                                {
                                    Some((f, e))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        match entities.len() {
                            // todo: do not make this an error if no entities are tested in testbench
                            0 => {
                                return Err(PlanError::TestbenchNoTest(
                                    local.get_key_by_index(b).unwrap().get_suffix().clone(),
                                ))
                            }
                            1 => Some(entities[0].0),
                            _ => {
                                return Err(PlanError::Ambiguous(
                                    "entities instantiated in the testbench".to_string(),
                                    entities
                                        .into_iter()
                                        .map(|f| {
                                            local
                                                .get_key_by_index(f.0)
                                                .unwrap()
                                                .get_suffix()
                                                .clone()
                                        })
                                        .collect(),
                                ))?
                            }
                        }
                    } else {
                        None
                    }
                }
            }
        };
        Ok((top, bench))
    }

    /// Modifies the `list` to only have a list of unique elements while preserving their original
    /// order.
    ///
    /// Removes all duplicate elements found after the first occurence of said element.
    fn remove_multi_occurences<T: Eq + Hash>(list: &Vec<T>) -> Vec<&T> {
        let mut result = Vec::new();
        // be prepared to store no more than the amount of elements in `list`
        result.reserve(list.len());
        // gradually build a set to track duplicates
        let mut set = HashSet::<&T>::new();

        for elem in list {
            if set.insert(elem) == true {
                result.push(elem)
            }
        }
        result
    }

    fn determine_file_order<'a>(
        global_graph: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        min_order: Vec<usize>,
    ) -> Vec<&'a IpFileNode<'a>> {
        // gather the files from each node in-order (multiple files can exist for a node)
        let file_order = {
            let mut file_map = HashMap::<String, (&IpFileNode, Vec<&HdlNode>)>::new();
            let mut file_order = Vec::<String>::new();

            let mut f_list = Vec::new();
            for i in &min_order {
                // access the node key and access the files associated with this key (the dependencies)
                let ipfs = global_graph
                    .get_node_by_index(*i)
                    .unwrap()
                    .as_ref()
                    .get_associated_files();

                ipfs.iter().for_each(|&e| {
                    let mut preds: Vec<&HdlNode> = global_graph
                        .predecessors(*i)
                        .into_iter()
                        .map(|e| e.1)
                        .collect();
                    match file_map.get_mut(e.get_file()) {
                        // merge dependencies together
                        Some((_file_node, deps)) => {
                            deps.append(&mut preds);
                        }
                        // log this node and its dependencies
                        None => {
                            file_order.push(e.get_file().clone());
                            file_map.insert(e.get_file().clone(), (e, preds));
                        }
                    }
                });
            }

            for file_name in &file_order {
                let (entry, deps) = file_map.get(file_name).unwrap();
                for &ifn in deps {
                    f_list
                        .append(&mut ifn.get_associated_files().into_iter().map(|i| *i).collect());
                }
                f_list.push(entry);
            }
            f_list
        };
        file_order
    }

    /// Filters out the local nodes existing within the current IP from the `global_graph`.
    pub fn compute_local_graph<'a>(
        global_graph: &'a GraphMap<CompoundIdentifier, HdlNode, ()>,
        working_lib: &Identifier,
        target: &Ip,
    ) -> GraphMap<&'a CompoundIdentifier, &'a HdlNode<'a>, &'a ()> {
        // restrict graph to units only found within the current IP
        let local_graph: GraphMap<&CompoundIdentifier, &HdlNode, &()> = global_graph
            .iter()
            // traverse subset of graph by filtering only for working library entities (current lib)
            .filter(|f| match f.0.get_prefix() {
                Some(iden) => iden == working_lib,
                None => false,
            })
            // filter by checking if the node's ip is the same as target
            .filter(|f| {
                let mut in_range: bool = true;
                for tag in f.1.get_associated_files() {
                    if tag.get_ip() != target {
                        in_range = false;
                        break;
                    }
                }
                in_range
            })
            .collect();

        local_graph
    }

    /// Performs the backend logic for creating a blueprint file (planning a design).
    fn run(
        &self,
        target: Ip,
        build_dir: &str,
        plug: Option<&Plugin>,
        catalog: Catalog,
    ) -> Result<(), Fault> {
        // create the build path to know where to begin storing files
        let mut build_path = target.get_root().clone();
        build_path.push(build_dir);

        // check if to clean the directory
        if self.clean == true && Path::exists(&build_path) == true {
            fs::remove_dir_all(&build_path)?;
        }

        // build entire ip graph and resolve with dynamic symbol transformation
        let ip_graph = algo::compute_final_ip_graph(&target, &catalog)?;

        // only write lockfile and exit if flag is raised
        if self.only_lock == true {
            Self::write_lockfile(&target, &ip_graph, self.force)?;
            return Ok(());
        }

        let files = algo::build_ip_file_list(&ip_graph);
        let global_graph = Self::build_full_graph(&files);

        let working_lib = Identifier::new_working();

        // restrict graph to units only found within the current IP
        let local_graph: GraphMap<&CompoundIdentifier, &HdlNode, &()> =
            Self::compute_local_graph(&global_graph, &working_lib, &target);

        let (top, bench) = match self.detect_bench(&global_graph, &local_graph, &working_lib) {
            Ok(r) => r,
            Err(e) => match e {
                PlanError::Ambiguous(_, _) => {
                    if self.all == true {
                        (None, None)
                    } else {
                        return Err(e)?;
                    }
                }
                _ => return Err(e)?,
            },
        };
        // determine the top-level node index
        let (top, bench) =
            match self.detect_top(&global_graph, &local_graph, &working_lib, top, bench) {
                Ok(r) => r,
                Err(e) => match e {
                    PlanError::Ambiguous(_, _) => {
                        if self.all == true {
                            (top, bench)
                        } else {
                            return Err(e)?;
                        }
                    }
                    _ => return Err(e)?,
                },
            };

        let top = match top {
            Some(i) => Some(Self::local_to_global(i, &global_graph, &local_graph).index()),
            None => None,
        };

        let bench = match bench {
            Some(i) => Some(Self::local_to_global(i, &global_graph, &local_graph).index()),
            None => None,
        };
        // guarantees top exists if not using --all

        // error if the user-defined top is not instantiated in the testbench. Say this can be fixed by adding '--all'
        if let Some(b) = &bench {
            // @idea: merge two topological sorted lists together by running top sort from bench and top sort from top if in this situation
            if self.all == false
                && global_graph
                    .get_graph()
                    .successors(top.unwrap())
                    .find(|i| i == b)
                    .is_none()
            {
                return Err(AnyError(format!("top unit '{}' is not tested in testbench '{}'\n\nIf you wish to continue, add the `--all` flag", global_graph.get_key_by_index(top.unwrap()).unwrap().get_suffix(), global_graph.get_key_by_index(*b).unwrap().get_suffix())))?;
            }
        }

        // [!] write the lock file
        Self::write_lockfile(&target, &ip_graph, true)?;

        // compute minimal topological ordering
        let min_order = match self.all {
            // perform topological sort on the entire graph
            true => {
                match local_graph.find_root() {
                    // only one topological sorting to compute
                    Ok(r) => {
                        let id = Self::local_to_global(r.index(), &global_graph, &local_graph);
                        global_graph
                            .get_graph()
                            .minimal_topological_sort(id.index())
                    }
                    // exclude roots that do not belong to the local graph
                    Err(rs) => {
                        let mut order = Vec::new();
                        rs.into_iter().for_each(|r| {
                            let id = Self::local_to_global(r.index(), &global_graph, &local_graph);
                            let mut subset = global_graph
                                .get_graph()
                                .minimal_topological_sort(id.index());
                            order.append(&mut subset);
                        });
                        order
                    }
                }
            }
            // perform topological sort on minimal subset of the graph
            false => {
                // determine which point is the upmost root
                let highest_point = match bench {
                    Some(b) => b,
                    None => match top {
                        Some(t) => t,
                        None => return Err(AnyError(format!("No top-level unit exists")))?,
                    },
                };
                global_graph
                    .get_graph()
                    .minimal_topological_sort(highest_point)
            }
        };

        // generate the file order while merging dependencies for common file path names together
        let file_order = Self::determine_file_order(&global_graph, min_order);

        // remove duplicate files from list while perserving order
        let file_order = Self::remove_multi_occurences(&file_order);

        // grab the names as strings
        let top_name = match top {
            Some(i) => global_graph
                .get_key_by_index(i)
                .unwrap()
                .get_suffix()
                .to_string(),
            None => String::new(),
        };
        let bench_name = match bench {
            Some(i) => global_graph
                .get_key_by_index(i)
                .unwrap()
                .get_suffix()
                .to_string(),
            None => String::new(),
        };

        // print information (maybe also print the plugin saved to .env too?)
        match top_name.is_empty() {
            false => println!("info: top-level set to {}", top_name.blue()),
            true => println!("{} no top-level set", "warning:".yellow()),
        }
        match bench_name.is_empty() {
            false => println!("info: testbench set to {}", bench_name.blue()),
            true => println!("{} no testbench set", "warning:".yellow()),
        }

        // store data in blueprint TSV format
        let mut blueprint_data = String::new();

        // [!] collect user-defined filesets
        {
            let current_files: Vec<String> =
                filesystem::gather_current_files(&target.get_root(), false);

            let mut vtable = VariableTable::new();
            // variables could potentially store empty strings if units are not set
            vtable.add("orbit.bench", &bench_name);
            vtable.add("orbit.top", &top_name);

            // store data in a map for quicker look-ups when comparing to plugin-defind filesets
            let mut cli_fset_map: HashMap<&String, &Fileset> = HashMap::new();

            // use command-line set filesets
            if let Some(fsets) = &self.filesets {
                for fset in fsets {
                    // insert into map structure
                    cli_fset_map.insert(fset.get_name(), &fset);
                }
            }

            // collect data for the given plugin
            if plug.is_some() == true && plug.unwrap().get_filesets().is_some() == true {
                for (name, pattern) in plug.unwrap().get_filesets().unwrap() {
                    let proper_key = Fileset::standardize_name(name);
                    // check if appeared in cli arguments
                    let (f_name, f_patt) = match cli_fset_map.contains_key(&proper_key) {
                        // override with fileset provided by command-line if conflicting names
                        true => {
                            // pull from map to ensure it is not double-counted when just writing command-line filesets
                            let entry = cli_fset_map.remove(&proper_key);
                            (name, entry.unwrap().get_pattern())
                        }
                        false => (name, pattern.inner()),
                    };
                    // perform variable substitution
                    let fset = Fileset::new()
                        .name(f_name)
                        .pattern(&variable::substitute(f_patt.to_string(), &vtable))?;
                    // match files
                    fset.collect_files(&current_files)
                        .into_iter()
                        .for_each(|f| {
                            blueprint_data += &fset.to_blueprint_string(&f);
                        });
                }
            }

            // check against every defined fileset in the command-line (call remaining filesets)
            for (_key, fset) in cli_fset_map {
                // perform variable substitution
                let fset = Fileset::new()
                    .name(fset.get_name())
                    .pattern(&variable::substitute(
                        fset.get_pattern().to_string(),
                        &vtable,
                    ))?;
                // match files
                fset.collect_files(&current_files)
                    .into_iter()
                    .for_each(|f| {
                        blueprint_data += &fset.to_blueprint_string(&f);
                    });
            }
        }

        // collect in-order HDL file list
        for file in file_order {
            if fileset::is_rtl(&file.get_file()) == true {
                blueprint_data +=
                    &format!("VHDL-RTL{0}{1}{0}{2}\n", BLUEPRINT_DELIMITER, file.get_library(), file.get_file());
            } else {
                blueprint_data +=
                    &format!("VHDL-SIM{0}{1}{0}{2}\n", BLUEPRINT_DELIMITER, file.get_library(), file.get_file());
            }
        }

        // create a output build directorie(s) if they do not exist
        if PathBuf::from(build_dir).exists() == false {
            fs::create_dir_all(build_dir).expect("could not create build dir");
        }

        // [!] create the blueprint file
        let blueprint_path = build_path.join(BLUEPRINT_FILE);
        let mut blueprint_file =
            File::create(&blueprint_path).expect("could not create blueprint file");
        // write the data
        blueprint_file
            .write_all(blueprint_data.as_bytes())
            .expect("failed to write data to blueprint");

        // create environment variables to .env file
        let mut envs = Environment::from_vec(vec![
            EnvVar::new().key(environment::ORBIT_TOP).value(&top_name),
            EnvVar::new()
                .key(environment::ORBIT_BENCH)
                .value(&bench_name),
        ]);
        // conditionally set the plugin used to plan
        match plug {
            Some(p) => {
                envs.insert(
                    EnvVar::new()
                        .key(environment::ORBIT_PLUGIN)
                        .value(&p.get_alias()),
                );
                ()
            }
            None => (),
        };
        environment::save_environment(&envs, &build_path)?;

        // create a blueprint file
        println!("info: Blueprint created at: {}", blueprint_path.display());
        Ok(())
    }
}

#[derive(Debug)]
pub enum PlanError {
    BadTestbench(Identifier),
    BadTop(Identifier),
    BadEntity(Identifier),
    TestbenchNoTest(Identifier),
    UnknownUnit(Identifier),
    UnknownEntity(Identifier),
    Ambiguous(String, Vec<Identifier>),
    Empty,
}

impl std::error::Error for PlanError {}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TestbenchNoTest(id) => write!(f, "no entities are tested in testbench {}", id),
            Self::UnknownEntity(id) => write!(f, "no entity named '{}' in the current ip", id),
            Self::Empty => write!(f, "no entities found"),
            Self::BadEntity(id) => write!(f, "primary design unit '{}' is not an entity", id),
            Self::BadTestbench(id) => write!(
                f,
                "entity '{}' is not a testbench and cannot be bench; use --top",
                id
            ),
            Self::BadTop(id) => write!(
                f,
                "entity '{}' is a testbench and cannot be top; use --bench",
                id
            ),
            Self::UnknownUnit(id) => {
                write!(f, "no primary design unit named '{}' in the current ip", id)
            }
            Self::Ambiguous(name, tbs) => write!(
                f,
                "multiple {} were found:\n {}",
                name,
                tbs.iter()
                    .fold(String::new(), |sum, x| { sum + &format!("    {}\n", x) })
            ),
        }
    }
}

const HELP: &str = "\
Generate a blueprint file.

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
    --lock-only             create the lockfile and exit
    --all                   include all found HDL files
    --force                 skip reading from the lock file

Use 'orbit help plan' to learn more about the command.
";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn remove_multi_occur() {
        // removes
        let arr = vec![1, 2, 2, 3, 4];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1, &2, &3, &4]);

        let arr = vec![1, 2, 2, 3, 4, 2, 1];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1, &2, &3, &4]);

        let arr = vec![1, 1, 1, 1, 1];
        assert_eq!(Plan::remove_multi_occurences(&arr), vec![&1]);

        // no changes
        let arr = vec![9, 8, 7, 6, 5, 4];
        assert_eq!(
            Plan::remove_multi_occurences(&arr),
            vec![&9, &8, &7, &6, &5, &4]
        );
    }
}
