use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::commands::plan::Plan;
use crate::core::vhdl::vhdl::Identifier;
use crate::util::anyerror::AnyError;

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
        let (graph, map) = Plan::build_graph(&files);

        let n = if let Some(ent) = &self.root {
            map.get(&ent).unwrap().index()
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