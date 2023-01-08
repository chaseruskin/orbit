use std::collections::HashMap;
use std::collections::HashSet;

use colored::Colorize;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::ip;
use crate::core::ip::IpFileNode;
use crate::core::manifest::IpManifest;
use crate::core::lang::vhdl::subunit::SubUnit;
use crate::core::lang::vhdl::symbol::CompoundIdentifier;
use crate::core::lang::vhdl::symbol::Entity;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::lang::vhdl::token::Identifier;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;

#[derive(Debug, PartialEq)]
enum IdentifierFormat {
    Long,
    Short
}

impl std::str::FromStr for IdentifierFormat {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(AnyError(format!("format can be 'long' or 'short'")))
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tree {
    root: Option<Identifier>,
    compress: bool,
    format: Option<IdentifierFormat>,
    ascii: bool,
    ip: bool,
}

impl FromCli for Tree {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Tree {
            root: cli.check_option(Optional::new("root").value("entity"))?,
            compress: cli.check_flag(Flag::new("compress"))?,
            ascii: cli.check_flag(Flag::new("ascii"))?,
            format: cli.check_option(Optional::new("format").value("fmt"))?,
            ip: cli.check_flag(Flag::new("ip"))?,
        });
        command
    }
}

impl Command for Tree {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // go to the ip directory
        c.goto_ip_path()?;

        if self.compress == true {
            todo!("compression logic")
        }

        // get the ip manifest
        let ip = IpManifest::from_path(c.get_ip_path().unwrap())?;

        // gather the catalog
        let catalog = Catalog::new()
            .store(c.get_store_path())
            .development(c.get_development_path().unwrap())?
            .installations(c.get_cache_path())?
            .available(c.get_vendors())?;
        
        self.run(ip, catalog)
    }
}

impl Tree {
    fn run(&self, target: IpManifest, catalog: Catalog) -> Result<(), Fault> {
        match &self.ip {
            true => self.run_ip_graph(target, catalog),
            false => self.run_hdl_graph(target, catalog),
        }
    }

    /// Construct and print the graph at an HDL-entity level.
    fn run_hdl_graph(&self, target: IpManifest, catalog: Catalog) -> Result<(), Fault> {
        let working_lib = Identifier::new_working();

        // build graph again but with entire set of all files available from all depdendencies
        let ip_graph = ip::compute_final_ip_graph(&target, &catalog)?;
        let files = ip::build_ip_file_list(&ip_graph);
        // build the complete graph
        let graph = Self::build_graph(&files);

        let n = if let Some(ent) = &self.root {
            // check if the identifier exists in the entity graph
            match graph.get_node_by_key(&CompoundIdentifier::new(working_lib, ent.clone())) {
                Some(id) => id.index(),
                None => return Err(PlanError::UnknownEntity(ent.clone()))?,
            }
        // auto-detect the root if possible
        } else {
            // traverse subset of graph by filtering only for working library entities
            let shallow_graph: GraphMap<&CompoundIdentifier, &EntityNode, &()> = graph.iter()
                .filter(|f| match f.0.get_prefix() { 
                    Some(iden) => iden == &working_lib, 
                    None => false } )
                .collect();
            match shallow_graph.find_root() {
                Ok(n) => graph.get_node_by_key(shallow_graph.get_key_by_index(n.index()).unwrap()).unwrap().index(),
                Err(e) => match e.len() {
                    0 => return Err(PlanError::Empty)?,
                    _ => return Err(PlanError::Ambiguous("roots".to_string(), e.into_iter().map(|f| { f.as_ref().entity.get_name().clone() }).collect()))?
                }
            }
        };

        let tree = graph.get_graph().treeview(n);
        for twig in &tree {
            let branch_str = match self.ascii {
                true => Self::to_ascii(&twig.0.to_string()),
                false => twig.0.to_string(),
            };
            println!("{}{}", branch_str, graph.get_node_by_index(twig.1).unwrap().as_ref().display(self.format.as_ref().unwrap_or(&IdentifierFormat::Short)));            
        }
        Ok(())
    }

    /// Construct and print the graph at an IP dependency level.
    fn run_ip_graph(&self, target: IpManifest, catalog: Catalog) -> Result<(), Fault> {
        let ip_graph = ip::compute_final_ip_graph(&target, &catalog)?;

        let tree = ip_graph.get_graph().treeview(0);
        for twig in &tree {
            println!("{}{}", twig.0, ip_graph.get_node_by_index(twig.1).unwrap().as_ref().as_ip().to_ip_spec());
        }
        Ok(())
    }

    /// Converts the original treeview text from using extended ascii characters
    /// to orginal ascii characters.
    fn to_ascii(s: &str) -> String {
        let mut transform = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            match c {
                '─' => transform.push('-'),
                '│' => transform.push('|'),
                '├' => transform.push('+'),
                '└' => transform.push('\\'),
                _ => transform.push(c),
            }
        }
        transform
    }

    /// Constructs a graph of the design heirarchy with entity nodes.
    fn build_graph<'a>(files: &'a Vec<IpFileNode>) -> GraphMap<CompoundIdentifier, EntityNode<'a>, ()> {
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut graph = GraphMap::<CompoundIdentifier, EntityNode, ()>::new();

        let mut sub_nodes: Vec<(Identifier, SubUnitNode)> = Vec::new();
        // store the (suffix, prefix) for all entities
        let mut component_pairs: HashMap<Identifier, Identifier> = HashMap::new();

        let mut package_identifiers: HashSet<Identifier> = HashSet::new();
        // read all files
        for source_file in files {
            // skip files that are not VHDL
            if crate::core::fileset::is_vhdl(&source_file.get_file()) == false {
                continue;
            }
            // parse VHDL code
            let contents = std::fs::read_to_string(&source_file.get_file()).unwrap();
            let symbols = symbol::VHDLParser::read(&contents).into_symbols();
            
            let lib = source_file.get_library();
            // add all entities to a graph and store architectures for later analysis
            symbols.into_iter()
                .for_each(|sym| {
                    match sym {
                        symbol::VHDLSymbol::Entity(e) => {
                            component_pairs.insert(e.get_name().clone(), lib.clone());
                            graph.add_node(CompoundIdentifier::new(lib.clone(), e.get_name().clone()), EntityNode::new(e, source_file));
                        },
                        symbol::VHDLSymbol::Architecture(arch) => {
                            sub_nodes.push((lib.clone(), SubUnitNode::new(SubUnit::from_arch(arch), source_file)));
                        },
                        symbol::VHDLSymbol::Configuration(cfg) => {
                            sub_nodes.push((lib.clone(), SubUnitNode::new(SubUnit::from_config(cfg), source_file)));
                        },
                        symbol::VHDLSymbol::Package(_) => {
                            package_identifiers.insert(sym.as_iden().unwrap().clone());
                        }
                        _ => (),
                    }
            });
        }

        // go through all subunits and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some((lib, node)) = sub_nodes_iter.next() {

            let node_name = CompoundIdentifier::new(lib, node.get_sub().get_entity().clone());

            // link to the owner and add subunit's source file
            // note: this also occurs in `plan.rs`
            let entity_node = match graph.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                // @todo: issue error because the entity (owner) is not declared
                None => continue
            };
            entity_node.as_ref_mut().add_file(node.get_file());
            // create edges
            for dep in node.get_sub().get_edges() {
                // verify we are not a package (will mismatch and make inaccurate graph)
                if package_identifiers.contains(dep.get_suffix()) == true {
                    continue;
                }
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        
                        let b = graph.add_edge_by_key(&CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()), &node_name, ());
                        match b {
                            // create black box entity
                            EdgeStatus::MissingSource => {
                                let dep_name = CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone());
                               
                                graph.add_node(dep_name.clone(), EntityNode::black_box(Entity::black_box(dep.get_suffix().clone())));
                                graph.add_edge_by_key(&dep_name, &node_name, ());
                            }
                            _ => ()
                        }
                    // this entity does not exist or was not logged
                    } else {
                        // create new node for black box entity
                        if graph.has_node_by_key(dep) == false {
                            graph.add_node(dep.clone(), EntityNode::black_box(Entity::black_box(dep.get_suffix().clone())));
                        }
                        graph.add_edge_by_key(&dep, &node_name, ());
                    }
                // the dependency has a prefix (a library) with it
                } else {
                    // verify we are not coming from a package (will mismatch and make inaccurate graph)
                    if package_identifiers.contains(dep.get_prefix().unwrap()) == true {
                        continue;
                    }
                    graph.add_edge_by_key(dep, &node_name, ());
                };
            }
        }
        graph
    }
}

use crate::core::lang::vhdl::symbol;
use crate::util::graph::EdgeStatus;
use crate::util::graphmap::GraphMap;

use super::plan::SubUnitNode;
use super::plan::PlanError;

#[derive(Debug, PartialEq)]
pub struct EntityNode<'a> {
    entity: symbol::Entity,
    files: Vec<&'a IpFileNode<'a>>,
}

impl<'a> EntityNode<'a> {
    
    fn new(entity: symbol::Entity, file: &'a IpFileNode<'a>) -> Self {
        let mut set = Vec::new();
        set.push(file);
        Self {
            entity: entity,
            files: set,
        }
    }

    fn is_black_box(&self) -> bool {
        self.files.is_empty()
    }

    fn black_box(entity: symbol::Entity) -> Self {
        Self { entity: entity, files: Vec::new() }
    }

    fn add_file(&mut self, file: &'a IpFileNode<'a>) {
        if self.files.contains(&file) == false {
            self.files.push(file);
        }
    }

    fn display(&self, fmt: &IdentifierFormat) -> String {
        if self.is_black_box() == true {
            format!("{} {}", self.entity.get_name().to_string().yellow(), "?".yellow())
        } else {
            match fmt {
                IdentifierFormat::Long => {
                    let ip = self.files.first().unwrap().get_ip_manifest();
                    format!("{} - {} v{}", self.entity.get_name(), ip.get_pkgid(), ip.get_version())
                }
                IdentifierFormat::Short => format!("{}", self.entity.get_name()),
            }
        }
    }
}

const HELP: &str = "\
View the hardware design hierarchy.

Usage:
    orbit tree [options]

Options:
    --root <entity>     top entity identifier to mark as the root node
    --compress          replace duplicate branches with a label marking
    --all               include all possible roots in tree
    --format <fmt>      select how to display entity names: 'long' or 'short'
    --ascii             use chars from the original 128 ascii set
    --ip                view the ip-level dependency graph

Use 'orbit help tree' to learn more about the command.
";