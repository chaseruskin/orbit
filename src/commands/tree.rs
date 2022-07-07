use std::collections::HashMap;

use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::commands::plan::Plan;
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

        self.run()
    }
}

impl Tree {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // gather all files
        let files = crate::core::fileset::gather_current_files(&std::env::current_dir().unwrap());
        // build the graph
        let (graph, map) = Self::build_graph(&files);

        let n = if let Some(ent) = &self.root {
            // check if the identifier exists in the entity graph
            if let Some(id) = map.get(&ent) {
                id.index()
            } else {
                return Err(AnyError(format!("entity '{}' does not exist in the current ip", ent)))?
            }
        } else {
            match graph.find_root() {
                Ok(n) => n,
                Err(e) => match e.len() {
                    0 => return Err(AnyError(format!("no entities found")))?,
                    1 => *e.first().unwrap(),
                    _ => {
                        // gather all identifier names
                        let mut roots = e.into_iter()
                            .map(|f| { graph.get_node(f).unwrap() });
                        let mut err_msg = String::from("multiple roots were found:\n");
                        while let Some(r) = roots.next() {
                            err_msg.push_str(&format!("\t{}\n", r));
                        }
                        return Err(AnyError(err_msg))?;
                    }
                }
            }
        };

        let tree = graph.treeview(n);
        for twig in &tree {
            let branch_str = match self.ascii {
                true => Self::to_ascii(&twig.0.to_string()),
                false => twig.0.to_string(),
            };
            println!("{}{}", branch_str, graph.get_node(twig.1).unwrap());
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


    fn build_graph(files: &Vec<String>) -> (Graph<Identifier, ()>, HashMap<Identifier, HashNode>) {
        // @TODO wrap graph in a hashgraph implementation
        let mut graph: Graph<Identifier, ()> = Graph::new();
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut map = HashMap::<Identifier, HashNode>::new();

        let mut archs: Vec<ArchitectureFile> = Vec::new();
        // read all files
        for source_file in files {
            if crate::core::fileset::is_vhdl(&source_file) == true {
                let contents = std::fs::read_to_string(&source_file).unwrap();
                let symbols = symbol::VHDLParser::read(&contents).into_symbols();
                // add all entities to a graph and store architectures for later analysis
                let mut iter = symbols.into_iter().filter_map(|f| {
                    match f {
                        symbol::VHDLSymbol::Entity(e) => Some(e),
                        symbol::VHDLSymbol::Architecture(arch) => {
                            archs.push(ArchitectureFile::new(arch, &source_file));
                            None
                        }
                        _ => None,
                    }
                });
                while let Some(e) = iter.next() {
                    let index = graph.add_node(e.get_name().clone());
                    let hn = HashNode::new(e, index, source_file.to_string());
                    map.insert(graph.get_node(index).unwrap().clone(), hn);
                }
            }
        }

        // go through all architectures and make the connections
        let mut archs = archs.into_iter();
        while let Some(af) = archs.next() {
            // link to the owner and add architecture's source file
            let entity_node = map.get_mut(&af.get_architecture().entity()).unwrap();
            entity_node.add_file(af.get_file().to_string());
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

#[derive(Debug, PartialEq)]
pub struct HashNode {
    index: usize,
    entity: symbol::Entity,
    files: Vec<String>,
}

impl HashNode {
    pub fn index(&self) -> usize {
        self.index
    }
    
    fn new(entity: symbol::Entity, index: usize, file: String) -> Self {
        let mut set = Vec::new();
        set.push(file);
        Self {
            entity: entity,
            index: index,
            files: set,
        }
    }

    fn add_file(&mut self, file: String) {
        if self.files.contains(&file) == false {
            self.files.push(file);
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