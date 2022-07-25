use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::ip;
use crate::core::ip::IpFileNode;
use crate::core::manifest::IpManifest;
use crate::core::vhdl::subunit::SubUnit;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::vhdl::token::Identifier;
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
            _ => Err(AnyError(format!("format must either be 'long' or 'short'")))
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
        // gather all files
        let current_files: Vec<IpFileNode> = crate::util::filesystem::gather_current_files(&std::env::current_dir().unwrap())
            .into_iter()
            .map(|f| IpFileNode::new(f, &target)).collect();

        // build the shallow graph
        let graph = Self::build_graph(&current_files);
        let n = if let Some(ent) = &self.root {
            // check if the identifier exists in the entity graph
            if let Some(id) = graph.get_node_by_key(&ent) {
                id.index()
            } else {
                return Err(PlanError::UnknownEntity(ent.clone()))?
            }
        } else {
            match graph.get_graph().find_root() {
                Ok(n) => n,
                Err(e) => match e.len() {
                    0 => return Err(PlanError::Empty)?,
                    1 => *e.first().unwrap(),
                    _ => return Err(PlanError::Ambiguous("roots".to_string(), e.into_iter().map(|f| { graph.get_key_by_index(f).unwrap().clone() }).collect()))?
                }
            }
        };

        // build graph again but with entire set of all files available from all depdendencies
        let ip_graph = ip::compute_final_ip_graph(&target, &catalog)?;
        let files = ip::build_ip_file_list(&ip_graph);
        // println!("{:?}", files.iter().map(|f| f.get_file()).collect::<Vec<&String>>());
        // remember the identifier to index transform to complete graph
        let iden = graph.get_key_by_index(n).unwrap();
        // build the complete graph
        let graph = Self::build_graph(&files);
        // transform the shallow's index number to the new graph's index number
        let n = graph.get_node_by_key(iden).unwrap().index();

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


    fn build_graph<'a>(files: &'a Vec<IpFileNode>) -> GraphMap<Identifier, HashNode<'a>, ()> {
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut graph = GraphMap::<Identifier, HashNode, ()>::new();

        let mut sub_nodes: Vec<SubUnitNode> = Vec::new();
        // read all files
        for source_file in files {
            if crate::core::fileset::is_vhdl(&source_file.get_file()) == true {
                let contents = std::fs::read_to_string(&source_file.get_file()).unwrap();
                let symbols = symbol::VHDLParser::read(&contents).into_symbols();
                // add all entities to a graph and store architectures for later analysis
                let mut iter = symbols.into_iter().filter_map(|f| {
                    match f {
                        symbol::VHDLSymbol::Entity(e) => Some(e),
                        symbol::VHDLSymbol::Architecture(arch) => {
                            sub_nodes.push(SubUnitNode::new(SubUnit::from_arch(arch), source_file));
                            None
                        },
                        symbol::VHDLSymbol::Configuration(cfg) => {
                            sub_nodes.push(SubUnitNode::new(SubUnit::from_config(cfg), source_file));
                            None
                        },
                        _ => None,
                    }
                });
                while let Some(e) = iter.next() {
                    graph.add_node(e.get_name().clone(), HashNode::new(e, source_file));
                }
            }
        }

        // go through all subunits and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some(node) = sub_nodes_iter.next() {
            // link to the owner and add subunit's source file
            let entity_node = graph.get_node_by_key_mut(&node.get_sub().get_entity()).unwrap();
            entity_node.as_ref_mut().add_file(node.get_file());
            // create edges
            for dep in node.get_sub().get_edges() {
                // verify the dep exists
                if graph.get_node_by_key(dep).is_some() {
                    graph.add_edge_by_key(dep, node.get_sub().get_entity(), ());
                }
            }
        }
        graph
    }
}

use crate::core::vhdl::symbol;
use crate::util::graphmap::GraphMap;

use super::plan::SubUnitNode;
use super::plan::PlanError;

#[derive(Debug, PartialEq)]
pub struct HashNode<'a> {
    entity: symbol::Entity,
    files: Vec<&'a IpFileNode<'a>>,
}

impl<'a> HashNode<'a> {
    
    fn new(entity: symbol::Entity, file: &'a IpFileNode<'a>) -> Self {
        let mut set = Vec::new();
        set.push(file);
        Self {
            entity: entity,
            files: set,
        }
    }

    fn add_file(&mut self, file: &'a IpFileNode<'a>) {
        if self.files.contains(&file) == false {
            self.files.push(file);
        }
    }

    fn display(&self, fmt: &IdentifierFormat) -> String {
        match fmt {
            IdentifierFormat::Long => {
                let ip = self.files.first().unwrap().get_ip_manifest();
                format!("{}  [{} {}]", self.entity.get_name(), ip.get_pkgid(), ip.get_version())
            }
            IdentifierFormat::Short => format!("{}", self.entity.get_name()),
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