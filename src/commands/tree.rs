use std::collections::HashMap;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::vhdl::token::Identifier;
use crate::util::anyerror::AnyError;
use crate::util::graph::Graph;

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
}

impl FromCli for Tree {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Tree {
            root: cli.check_option(Optional::new("root").value("entity"))?,
            compress: cli.check_flag(Flag::new("compress"))?,
            ascii: cli.check_flag(Flag::new("ascii"))?,
            format: cli.check_option(Optional::new("format").value("fmt"))?,
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
            .available(&&c.get_vendor_path())?;

        self.run(ip, catalog)
    }
}

impl Tree {
    fn run(&self, target: IpManifest, catalog: Catalog) -> Result<(), Box<dyn std::error::Error>> {
        // gather all files
        let current_files: Vec<IpFileNode> = crate::util::filesystem::gather_current_files(&std::env::current_dir().unwrap())
            .into_iter()
            .map(|f| IpFileNode::new(f, &target)).collect();

        // build the shallow graph
        let (graph, map) = Self::build_graph(&current_files);

        let n = if let Some(ent) = &self.root {
            // check if the identifier exists in the entity graph
            if let Some(id) = map.get(&ent) {
                id.index()
            } else {
                return Err(PlanError::UnknownEntity(ent.clone()))?
            }
        } else {
            match graph.find_root() {
                Ok(n) => n,
                Err(e) => match e.len() {
                    0 => return Err(PlanError::Empty)?,
                    1 => *e.first().unwrap(),
                    _ => return Err(PlanError::Ambiguous("roots".to_string(), e.into_iter().map(|f| { graph.get_node(f).unwrap().clone() }).collect()))?
                }
            }
        };

        // build graph again but with entire set of all files available from all depdendencies
        let build_list = Plan::resolve_dependencies(&target, &catalog)?;
        let files = Plan::assemble_all_files(build_list);
        // remember the identifier to index transform to complete graph
        let iden = graph.get_node(n).unwrap();
        // build the complete graph
        let (graph, map) = Self::build_graph(&files);
        // transform the shallow's index number to the new graph's index number
        let n = map.get(iden).unwrap().index();

        let tree = graph.treeview(n);
        for twig in &tree {
            let branch_str = match self.ascii {
                true => Self::to_ascii(&twig.0.to_string()),
                false => twig.0.to_string(),
            };
            println!("{}{}", branch_str, map.get(graph.get_node(twig.1).unwrap()).unwrap().display(self.format.as_ref().unwrap_or(&IdentifierFormat::Short)));            
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


    fn build_graph<'a>(files: &'a Vec<IpFileNode>) -> (Graph<Identifier, ()>, HashMap<Identifier, HashNode<'a>>) {
        // @TODO wrap graph in a hashgraph implementation
        let mut graph: Graph<Identifier, ()> = Graph::new();
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut map = HashMap::<Identifier, HashNode>::new();

        let mut archs: Vec<ArchitectureFile> = Vec::new();
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
                            archs.push(ArchitectureFile::new(arch, source_file));
                            None
                        }
                        _ => None,
                    }
                });
                while let Some(e) = iter.next() {
                    let index = graph.add_node(e.get_name().clone());
                    let hn = HashNode::new(e, index, source_file);
                    map.insert(graph.get_node(index).unwrap().clone(), hn);
                }
            }
        }

        // go through all architectures and make the connections
        let mut archs = archs.into_iter();
        while let Some(af) = archs.next() {
            // link to the owner and add architecture's source file
            let entity_node = map.get_mut(&af.get_architecture().entity()).unwrap();
            entity_node.add_file(af.get_file());
            // create edges
            for dep in af.get_architecture().edges() {
                // verify the dep exists
                if let Some(node) = map.get(dep) {
                    graph.add_edge(node.index(), map.get(af.get_architecture().entity()).unwrap().index(), ());
                }
            }
        }
        (graph, map)
    }
}

use crate::core::vhdl::symbol;

use super::plan::ArchitectureFile;
use super::plan::IpFileNode;
use super::plan::Plan;
use super::plan::PlanError;

#[derive(Debug, PartialEq)]
pub struct HashNode<'a> {
    index: usize,
    entity: symbol::Entity,
    files: Vec<&'a IpFileNode<'a>>,
}

impl<'a> HashNode<'a> {
    pub fn index(&self) -> usize {
        self.index
    }
    
    fn new(entity: symbol::Entity, index: usize, file: &'a IpFileNode<'a>) -> Self {
        let mut set = Vec::new();
        set.push(file);
        Self {
            entity: entity,
            index: index,
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

Use 'orbit help tree' to learn more about the command.
";