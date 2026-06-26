//
//  Copyright (C) 2022-2026  Chase Ruskin
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

use crate::commands::helps::analyze;
use crate::commands::plan;
use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::core::lang::node::HdlNode;
use crate::core::lang::node::HdlSymbol;
use crate::core::lang::node::SubUnitNode;
use crate::core::lang::reference::CompoundIdentifier;
use crate::core::project::Project;
use crate::util::anyerror::Fault;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::PRJ_CATALOG_EX_LOCK_NAME;
use crate::util::graphmap::GraphMap;
use serde_derive::Serialize;
use std::collections::HashMap;

use cliproc::{Arg, Cli, Help, Subcommand};
use cliproc::{cli, proc, stage::*};

#[derive(Debug, PartialEq, Serialize)]
struct RefInfo {
    library: Option<String>,
    name: String,
}

#[derive(Debug, PartialEq, Serialize)]
struct UnitInfo {
    name: String,
    unit_type: String,
    language: String,
    library: String,
    sources: Vec<String>,
    refs: Vec<RefInfo>,
}

#[derive(Debug, PartialEq, Serialize)]
struct AnalyzeOutput {
    units: Vec<UnitInfo>,
}

#[derive(Debug, PartialEq)]
pub struct Analyze {
    json: bool,
    all: bool,
}

/// Total order over [`RefInfo`]: library first (empty before any named
/// library), then unit name. Pulled out so the analyze output stays
/// byte-for-byte deterministic regardless of `HashSet` iteration order —
/// downstream `gazelle_orbit` users rely on the BUILD files being
/// reproducible across runs (gazelle's CI step `git diff --exit-code`
/// catches any drift).
fn ref_order(a: &RefInfo, b: &RefInfo) -> std::cmp::Ordering {
    a.library
        .as_deref()
        .unwrap_or("")
        .cmp(b.library.as_deref().unwrap_or(""))
        .then(a.name.cmp(&b.name))
}

impl Subcommand<Context> for Analyze {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(analyze::HELP))?;
        Ok(Analyze {
            json: cli.check(Arg::flag("json"))?,
            all: cli.check(Arg::flag("all"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        c.jump_to_working_project()?;

        let project = Project::load(c.get_project_path().unwrap().clone(), true, true, false)?;

        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;
        plan::resolve_missing_deps(c, &project, &mut catalog, false)?;
        let catalog = catalog;

        let result = self.run(project, catalog, c.are_units_private_by_default());
        crate::util::filesystem::release_lock(&cache_ap_lock)?;
        result
    }
}

impl Analyze {
    fn run(&self, target: Project, catalog: Catalog, priv_by_def: bool) -> Result<(), Fault> {
        let project_graph =
            algo::compute_final_project_graph(&target, Some(&catalog), priv_by_def)?;
        let files = algo::build_project_file_list(&project_graph, &target);

        let graph = Self::build_unit_graph(&files)?;

        let mut output = AnalyzeOutput {
            units: Vec::new(),
        };

        let local_filter: Option<Vec<CompoundIdentifier>> = if self.all {
            None
        } else {
            let local_graph = Plan::compute_local_graph(&graph, &target, true);
            Some(
                local_graph
                    .iter()
                    .map(|(k, _, _)| (*k).clone())
                    .collect(),
            )
        };

        for (key, node, _idx) in graph.iter() {
            if node.is_black_box() {
                continue;
            }
            if let Some(filter) = &local_filter {
                if !filter.contains(key) {
                    continue;
                }
            }

            let sym = node.get_symbol();
            let name = sym.get_name().to_string();
            let library = node.get_library().to_string();
            let language = node.get_lang().to_string();
            let unit_type = Self::symbol_type_name(sym);

            let sources: Vec<String> = node
                .get_associated_files()
                .iter()
                .map(|f| f.get_file().clone())
                .collect();

            let refs: Vec<RefInfo> = match sym.get_refs() {
                Some(ref_set) => {
                    let mut refs: Vec<RefInfo> = ref_set
                        .iter()
                        .map(|r| RefInfo {
                            library: r.get_prefix().map(|p| p.to_string()),
                            name: r.get_suffix().to_string(),
                        })
                        .collect();
                    refs.sort_by(ref_order);
                    refs
                }
                None => Vec::new(),
            };

            output.units.push(UnitInfo {
                name,
                unit_type,
                language,
                library,
                sources,
                refs,
            });
        }

        output.units.sort_by(|a, b| a.name.cmp(&b.name));

        if self.json {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        } else {
            for unit in &output.units {
                println!(
                    "{} {} ({}) [{}]",
                    unit.unit_type, unit.name, unit.library, unit.language
                );
                for src in &unit.sources {
                    println!("  src: {}", src);
                }
                for r in &unit.refs {
                    match &r.library {
                        Some(lib) => println!("  ref: {}.{}", lib, r.name),
                        None => println!("  ref: {}", r.name),
                    }
                }
            }
        }

        Ok(())
    }

    fn symbol_type_name(sym: &HdlSymbol) -> String {
        match sym {
            HdlSymbol::Vhdl(v) => {
                if v.as_entity().is_some() {
                    "entity"
                } else if v.as_package().is_some() {
                    "package"
                } else if v.as_context().is_some() {
                    "context"
                } else if v.as_configuration().is_some() {
                    "configuration"
                } else {
                    "unknown"
                }
            }
            HdlSymbol::Verilog(v) => {
                if v.as_module().is_some() {
                    "module"
                } else {
                    "config"
                }
            }
            HdlSymbol::SystemVerilog(v) => {
                if v.as_module().is_some() {
                    "module"
                } else {
                    "config"
                }
            }
            HdlSymbol::BlackBox(_) => "blackbox",
        }
        .to_string()
    }

    fn build_unit_graph<'a>(
        files: &'a Vec<crate::core::algo::ProjectFileNode>,
    ) -> Result<GraphMap<CompoundIdentifier, HdlNode<'a>, ()>, Fault> {
        let mut graph_map = GraphMap::<CompoundIdentifier, HdlNode, ()>::new();
        let mut sub_nodes: Vec<(LangIdentifier, SubUnitNode)> = Vec::new();
        let mut component_pairs: HashMap<LangIdentifier, LangIdentifier> = HashMap::new();

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

        // Merge subunit source files and refs into their parent entities
        for (lang_lib, node) in sub_nodes {
            let hdl_lib = match lang_lib.as_vhdl_name() {
                Some(lib) => lib,
                None => continue,
            };
            let node_name =
                CompoundIdentifier::new_vhdl(hdl_lib.clone(), node.get_sub().get_entity().clone());

            let entity_node = match graph_map.get_node_by_key_mut(&node_name) {
                Some(en) => en,
                None => continue,
            };

            entity_node.as_ref_mut().add_file(node.get_file());

            let sub_refs = node.get_sub().get_refs().clone();
            entity_node.as_ref_mut().get_symbol_mut().inherit_refs(sub_refs);
        }

        Ok(graph_map)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Locks the JSON schema that `gazelle_orbit` (and any other
    /// downstream consumer of `orbit analyze --json`) reads. Any field
    /// rename, type change, or null-handling change should fail this test
    /// loudly rather than silently breaking a downstream tool.
    #[test]
    fn json_schema_shape() {
        let out = AnalyzeOutput {
            units: vec![UnitInfo {
                name: "nand_gate".to_string(),
                unit_type: "entity".to_string(),
                language: "vhdl".to_string(),
                library: "gates".to_string(),
                sources: vec!["src/nand_gate.vhd".to_string()],
                refs: vec![
                    RefInfo {
                        library: Some("ieee".to_string()),
                        name: "std_logic_1164".to_string(),
                    },
                    RefInfo {
                        library: None,
                        name: "inverter".to_string(),
                    },
                ],
            }],
        };
        let s = serde_json::to_string(&out).unwrap();
        // Walk the parsed JSON instead of string-matching so trivial
        // whitespace differences don't break the assertion.
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let unit = &v["units"][0];
        assert_eq!(unit["name"], "nand_gate");
        assert_eq!(unit["unit_type"], "entity");
        assert_eq!(unit["language"], "vhdl");
        assert_eq!(unit["library"], "gates");
        assert_eq!(unit["sources"][0], "src/nand_gate.vhd");
        // Library-qualified ref keeps its library string.
        assert_eq!(unit["refs"][0]["library"], "ieee");
        assert_eq!(unit["refs"][0]["name"], "std_logic_1164");
        // Unqualified ref serializes library as JSON null (not omitted,
        // not empty string) — gazelle distinguishes "no library" from
        // "library == ''".
        assert!(unit["refs"][1]["library"].is_null());
        assert_eq!(unit["refs"][1]["name"], "inverter");
    }

    /// Deterministic ordering of refs is the contract that keeps the
    /// generated BUILD files stable across runs. Sort key: library first
    /// (None < any Some), then unit name.
    #[test]
    fn refs_sort_library_then_name() {
        let mut refs = vec![
            RefInfo {
                library: Some("work".to_string()),
                name: "b".to_string(),
            },
            RefInfo {
                library: None,
                name: "z".to_string(),
            },
            RefInfo {
                library: Some("ieee".to_string()),
                name: "std_logic_1164".to_string(),
            },
            RefInfo {
                library: Some("work".to_string()),
                name: "a".to_string(),
            },
            RefInfo {
                library: None,
                name: "a".to_string(),
            },
        ];
        refs.sort_by(ref_order);
        let got: Vec<(Option<String>, String)> = refs
            .into_iter()
            .map(|r| (r.library, r.name))
            .collect();
        assert_eq!(
            got,
            vec![
                (None, "a".to_string()),
                (None, "z".to_string()),
                (Some("ieee".to_string()), "std_logic_1164".to_string()),
                (Some("work".to_string()), "a".to_string()),
                (Some("work".to_string()), "b".to_string()),
            ]
        );
    }

    /// Black-box symbols are the only `HdlSymbol` variant constructible
    /// without parsing real HDL — covering them confirms the match
    /// statement's fallback. The other variants are exercised
    /// transitively whenever orbit runs against real source.
    #[test]
    fn symbol_type_blackbox() {
        let sym = HdlSymbol::BlackBox("foo".to_string());
        assert_eq!(Analyze::symbol_type_name(&sym), "blackbox");
    }
}
