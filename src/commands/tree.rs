//
//  Copyright (C) 2022-2025  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use super::plan::PlanError;
use crate::commands::helps::tree;
use crate::commands::plan;
use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::algo::ProjectFileNode;
use crate::core::algo::ProjectNode;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::lang::node::HdlNode;
use crate::core::lang::node::HdlSymbol;
use crate::core::lang::node::IdentifierFormat;
use crate::core::lang::node::SubUnitNode;
use crate::core::lang::reference::CompoundIdentifier;
use crate::core::lang::vhdl::token::Identifier as VhdlIdentifier;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::core::project::Project;
use crate::core::project::ProjectIdSpec;
use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::Fault;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;
use crate::util::graph::EdgeStatus;
use crate::util::graphmap::GraphMap;
use colored::ColoredString;
use colored::Colorize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(PartialEq, Debug, Serialize)]
struct SerNode {
    name: String,
    targets: Vec<String>,
    sources: Vec<String>,
}

impl SerNode {
    pub fn from_hdl_graph(
        graph: &GraphMap<CompoundIdentifier, HdlNode<'_>, ()>,
        id: usize,
    ) -> Self {
        let name = graph
            .get_node_by_index(id)
            .unwrap()
            .as_ref()
            .get_symbol()
            .get_name()
            .to_string();
        let sources = graph
            .predecessors(id)
            .into_iter()
            .map(|i| i.1.get_symbol().get_name().to_string())
            .collect();
        let targets = graph
            .successors(id)
            .into_iter()
            .map(|i| i.1.get_symbol().get_name().to_string())
            .collect();
        Self {
            name: name,
            sources: sources,
            targets: targets,
        }
    }

    pub fn from_project_graph(
        graph: &GraphMap<ProjectIdSpec, ProjectNode<'_>, ()>,
        id: usize,
    ) -> Self {
        let name = graph
            .get_node_by_index(id)
            .unwrap()
            .as_ref()
            .as_project()
            .get_man()
            .get_project()
            .get_name()
            .to_string();
        let sources = graph
            .predecessors(id)
            .into_iter()
            .map(|i| {
                i.1.as_project()
                    .get_man()
                    .get_project()
                    .get_name()
                    .to_string()
            })
            .collect();
        let targets = graph
            .successors(id)
            .into_iter()
            .map(|i| {
                i.1.as_project()
                    .get_man()
                    .get_project()
                    .get_name()
                    .to_string()
            })
            .collect();
        Self {
            name: name,
            sources: sources,
            targets: targets,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Kind {
    Unit,
    Project,
    All,
}

impl FromStr for Kind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unit" => Ok(Kind::Unit),
            "project" => Ok(Kind::Project),
            "all" => Ok(Kind::All),
            _ => Err(Error::EdgeKindInvalid(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Charset {
    Utf8,
    Ascii,
}

impl FromStr for Charset {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "utf8" => Ok(Charset::Utf8),
            "ascii" => Ok(Charset::Ascii),
            _ => Err(Error::CharsetInvalid(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Tree {
    roots: Option<Vec<VhdlIdentifier>>,
    no_dedupe: bool,
    invert: bool,
    depth: Option<usize>,
    format: Option<IdentifierFormat>,
    charset: Charset,
    edges: Kind,
    json: bool,
}

impl Subcommand<Context> for Tree {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(tree::HELP))?;
        Ok(Tree {
            json: cli.check(Arg::flag("json"))?,
            invert: cli.check(Arg::flag("invert").switch('i'))?,
            no_dedupe: cli.check(Arg::flag("no-dedupe"))?,
            depth: cli.get(Arg::option("depth").value("depth"))?,
            edges: cli
                .get(Arg::option("edges").switch('e').value("kind"))?
                .unwrap_or(Kind::Unit),
            charset: cli
                .get(Arg::option("charset").value("charset"))?
                .unwrap_or(Charset::Utf8),
            format: cli.get(Arg::option("format").value("format"))?,
            roots: cli.get_all(Arg::positional("unit"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // go to the ip directory
        c.jump_to_working_project()?;

        // get the ip manifest
        let project = Project::load(c.get_project_path().unwrap().clone(), true, false)?;

        // before we gather the catalog, request an "APPEND" action to the cache
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // gather the catalog and resolve any missing dependencies
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;
        let catalog = plan::resolve_missing_deps(c, &project, catalog, false)?;

        let result = self.run(project, catalog, c.are_units_private_by_default());
        // release our "APPEND" action to the cache
        crate::util::filesystem::release_lock(&cache_ap_lock)?;
        result
    }
}

impl Tree {
    fn run(&self, target: Project, catalog: Catalog, priv_by_def: bool) -> Result<(), Fault> {
        // Determine how to display the dependencies for the project
        match &self.edges {
            Kind::Unit => self.run_hdl_graph(target, catalog, true, priv_by_def),
            Kind::Project => self.run_project_graph(target, catalog, priv_by_def),
            Kind::All => self.run_hdl_graph(target, catalog, false, priv_by_def),
        }
    }

    /// Construct and print the graph at an HDL-entity level.
    ///
    /// If `only_modules` is true, then the tree will only report back entity/module instantiations
    /// within other entity/modules. If false, then any type of primary design unit reference will
    /// be included.
    fn run_hdl_graph(
        &self,
        target: Project,
        catalog: Catalog,
        only_modules: bool,
        priv_by_def: bool,
    ) -> Result<(), Fault> {
        let working_lib = target.get_hdl_library();

        // build graph again but with entire set of all files available from all depdendencies
        let project_graph =
            algo::compute_final_project_graph(&target, Some(&catalog), priv_by_def)?;
        let files = algo::build_project_file_list(&project_graph, &target);

        // build the complete graph (using entities as the nodes)
        let global_graph = Self::build_graph(&files, only_modules)?;

        let roots = match &self.roots {
            Some(user_roots) => {
                // restrict graph to units only found within the current project
                let local_graph = Plan::compute_local_graph(&global_graph, &target);
                let mut roots = Vec::new();
                for root_name in user_roots {
                    // check if the identifier exists in the entity graph
                    let i = match local_graph.get_node_by_key(&&CompoundIdentifier::new(
                        working_lib.clone(),
                        LangIdentifier::Vhdl(root_name.clone()),
                    )) {
                        Some(id) => {
                            // verify the unit is a component
                            if id.as_ref().get_symbol().is_component() == false {
                                return Err(PlanError::BadEntity(root_name.clone()))?;
                            }
                            id.index()
                        }
                        None => {
                            return Err(Error::GetUnitNotFound(
                                root_name.to_string(),
                                Hint::ShowAvailableUnitsLocal,
                            ))?
                        }
                    };
                    roots.push(Plan::local_to_global(i, &global_graph, &local_graph).index())
                }
                roots
            }
            None => {
                // restrict graph to units only found within the current project
                let local_graph = Plan::compute_local_graph(&global_graph, &target);
                // compile list of all roots
                let mut roots = Vec::new();
                if self.invert == true {
                    local_graph.find_leaves().into_iter().for_each(|f| {
                        roots.push(Plan::local_to_global(f, &global_graph, &local_graph).index())
                    })
                } else {
                    match local_graph.find_root() {
                        Ok(i) => roots.push(
                            Plan::local_to_global(i.index(), &global_graph, &local_graph).index(),
                        ),
                        Err(e) => match e.len() {
                            0 => return Err(PlanError::Empty)?,
                            _ => e.into_iter().for_each(|f| {
                                roots.push(
                                    Plan::local_to_global(f, &global_graph, &local_graph).index(),
                                )
                            }),
                        },
                    }
                }
                roots
            }
        };

        // turn on de-duplication when asking for json or when not requesting no deduplication
        let en_dedupe = self.json || !self.no_dedupe;

        // serialized nodes for json output
        let mut ser_nodes = Vec::new();

        // display each root's tree to the console
        roots
            .iter()
            .filter(|k| {
                global_graph
                    .get_node_by_index(**k)
                    .unwrap()
                    .as_ref()
                    .get_symbol()
                    .is_component()
                    || only_modules == false
            })
            .for_each(|n| {
                let tree =
                    global_graph
                        .get_graph()
                        .treeview(*n, en_dedupe, self.invert, self.depth);
                for twig in &tree {
                    let branch_str = match self.charset == Charset::Ascii {
                        true => Self::to_ascii(&twig.0.to_string()),
                        false => twig.0.to_string(),
                    };
                    match self.json {
                        true => {
                            // only add if not a duplicate!
                            if twig.0.is_dupe() == false {
                                ser_nodes.push(SerNode::from_hdl_graph(&global_graph, twig.1))
                            }
                        }
                        false => {
                            println!(
                                "{}{}{}",
                                branch_str,
                                global_graph
                                    .get_node_by_index(twig.1)
                                    .unwrap()
                                    .as_ref()
                                    .display(
                                        self.format.as_ref().unwrap_or(&IdentifierFormat::Short)
                                    ),
                                if twig.0.is_dupe() == true {
                                    " (*)".blue()
                                } else {
                                    ColoredString::default()
                                },
                            );
                        }
                    }
                }
            });

        if self.json == true {
            println!("{}", serde_json::to_string(&ser_nodes).unwrap());
        }
        Ok(())
    }

    /// Construct and print the graph at a project dependency level.
    fn run_project_graph(
        &self,
        target: Project,
        catalog: Catalog,
        priv_by_def: bool,
    ) -> Result<(), Fault> {
        let project_graph =
            algo::compute_final_project_graph(&target, Some(&catalog), priv_by_def)?;

        // turn on de-duplication when asking for json or when not requesting no deduplication
        let en_dedupe = self.json || !self.no_dedupe;

        let tree = project_graph
            .get_graph()
            .treeview(0, en_dedupe, self.invert, self.depth);

        let mut ser_nodes = Vec::new();

        for twig in &tree {
            let branch_str = match self.charset == Charset::Ascii {
                true => Self::to_ascii(&twig.0.to_string()),
                false => twig.0.to_string(),
            };
            match self.json {
                true => {
                    // only add if not a duplicate!
                    if twig.0.is_dupe() == false {
                        ser_nodes.push(SerNode::from_project_graph(&project_graph, twig.1))
                    }
                }
                false => {
                    println!(
                        "{}{}",
                        branch_str,
                        project_graph
                            .get_node_by_index(twig.1)
                            .unwrap()
                            .as_ref()
                            .as_project()
                            .get_man()
                            .get_project()
                            .into_project_id_spec()
                    );
                }
            }
        }

        if self.json == true {
            println!("{}", serde_json::to_string(&ser_nodes).unwrap());
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
    fn build_graph<'a>(
        files: &'a Vec<ProjectFileNode>,
        only_modules: bool,
    ) -> Result<GraphMap<CompoundIdentifier, HdlNode<'a>, ()>, Fault> {
        // entity identifier, HashNode (hash-node holds entity structs)
        let mut graph_map = GraphMap::<CompoundIdentifier, HdlNode, ()>::new();

        let mut sub_nodes: Vec<(LangIdentifier, SubUnitNode)> = Vec::new();
        // store the (suffix, prefix) for all entities
        let mut component_pairs: HashMap<LangIdentifier, LangIdentifier> = HashMap::new();

        // read all files (same as planning)
        for source_file in files {
            match source_file.get_language() {
                Lang::Vhdl => Plan::create_vhdl_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                    &mut sub_nodes,
                )?,
                Lang::Verilog => {
                    Plan::create_verilog_node(&mut graph_map, source_file, &mut component_pairs)?
                }
                Lang::SystemVerilog => Plan::create_systemverilog_node(
                    &mut graph_map,
                    source_file,
                    &mut component_pairs,
                )?,
            }
        }

        // differs from planning below

        // add edges according to verilog
        Plan::connect_edges_from_verilog(&mut graph_map, &mut component_pairs, only_modules);

        // go through all subunits and make the connections
        let mut sub_nodes_iter = sub_nodes.into_iter();
        while let Some((lang_lib, node)) = sub_nodes_iter.next() {
            let hdl_lib = lang_lib.as_vhdl_name().unwrap();
            let node_name =
                CompoundIdentifier::new_vhdl(hdl_lib.clone(), node.get_sub().get_entity().clone());

            // link to the owner and add subunit's source file
            // note: this also occurs in `plan.rs`
            let entity_node = match graph_map.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                // @TODO: issue error because the entity (owner) is not declared
                None => continue,
            };
            entity_node.as_ref_mut().add_file(node.get_file());
            // grab the list of the primary design unit's references to be used when getting "all" dependencies
            let pri_node = entity_node.as_ref().get_symbol().copy_refs();

            // create edges by ordered edge list (for entities)
            let edges = match only_modules {
                true => node.get_sub().get_edge_list_entities(),
                false => {
                    // add the primary design unit's references as well
                    let mut edges = Vec::new();
                    if let Some(set) = &pri_node {
                        edges.extend(set);
                    }
                    edges.extend(node.get_sub().get_edge_list().clone());
                    edges
                }
            };

            for dep in edges {
                // need to locate the key with a suffix matching `dep` if it was a component instantiation
                if dep.get_prefix().is_none() {
                    if let Some(lib) = component_pairs.get(dep.get_suffix()) {
                        let b = graph_map.add_edge_by_key(
                            &CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone()),
                            &node_name,
                            (),
                        );
                        match b {
                            // create black box entity
                            EdgeStatus::MissingSource => {
                                let dep_name =
                                    CompoundIdentifier::new(lib.clone(), dep.get_suffix().clone());

                                graph_map.add_node(
                                    dep_name.clone(),
                                    HdlNode::black_box(HdlSymbol::BlackBox(
                                        dep.get_suffix().to_string(),
                                    )),
                                );
                                graph_map.add_edge_by_key(&dep_name, &node_name, ());
                            }
                            _ => (),
                        }
                    // this entity does not exist or was not logged
                    } else {
                        // create new node for black box entity
                        if graph_map.has_node_by_key(dep) == false {
                            graph_map.add_node(
                                dep.clone(),
                                HdlNode::black_box(HdlSymbol::BlackBox(
                                    dep.get_suffix().to_string(),
                                )),
                            );
                        }
                        graph_map.add_edge_by_key(&dep, &node_name, ());
                    }
                // the dependency has a prefix (a library) with it
                } else {
                    let local_lib_id = CompoundIdentifier::new(
                        LangIdentifier::Vhdl(hdl_lib.clone()),
                        dep.get_suffix().clone(),
                    );
                    // if the prefix is "work", replace it with the current unit's hdl library
                    let resolved_dep_id = match dep.get_prefix().unwrap() {
                        LangIdentifier::Vhdl(i) => {
                            if i == &VhdlIdentifier::new_working() {
                                &local_lib_id
                            } else {
                                dep
                            }
                        }
                        _ => dep,
                    };
                    graph_map.add_edge_by_key(&resolved_dep_id, &node_name, ());
                };
            }
        }

        Ok(graph_map)
    }
}
