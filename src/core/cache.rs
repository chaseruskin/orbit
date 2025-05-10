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

use crate::commands::plan::Plan;
use crate::core::algo;
use crate::core::ip::Ip;
use crate::util::anyerror::Fault;
use serde_derive::{Deserialize, Serialize};

use super::lang::LangIdentifier;
use crate::core::lang::node::HdlNode;
use crate::core::lang::{Lang, LangUnit};
use crate::core::visibility::Visibility;

use std::str::FromStr;

type VhdlIdentifier = crate::core::lang::vhdl::token::identifier::Identifier;
type VerilogIdentifier = crate::core::lang::verilog::token::identifier::Identifier;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitCache {
    identifier: String,
    #[serde(rename = "type")]
    symbol: String,
    language: Lang,
    visibility: Visibility,
    sources: Vec<String>,
    dependencies: Vec<String>,
}

impl UnitCache {
    pub fn from_graph_entry(
        ip: &Ip,
        name: LangIdentifier,
        node: &HdlNode,
        unit: &LangUnit,
        deps: Vec<&LangIdentifier>,
    ) -> Self {
        let base_path_offset = ip.get_root().as_os_str().len();

        Self {
            identifier: name.to_string(),
            symbol: unit.to_string(),
            language: unit.get_lang(),
            visibility: unit.get_visibility().clone(),
            sources: node
                .get_associated_files()
                .iter()
                .map(|f| {
                    let abs_path = f.get_file();
                    // +1 to include removing the final '/' to make the path relative
                    abs_path.get(base_path_offset + 1..).unwrap().to_string()
                })
                .collect(),
            dependencies: deps.into_iter().map(|f| f.to_string()).collect(),
        }
    }

    pub fn get_sources(&self) -> &Vec<String> {
        &self.sources
    }

    pub fn get_lang(&self) -> Lang {
        self.language.clone()
    }

    pub fn get_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn get_visibility(&self) -> Visibility {
        self.visibility.clone()
    }

    pub fn get_name(&self) -> LangIdentifier {
        match self.get_lang() {
            Lang::SystemVerilog | Lang::Verilog => {
                LangIdentifier::from(VerilogIdentifier::from_str(&self.identifier).unwrap())
            }
            Lang::Vhdl => LangIdentifier::from(VhdlIdentifier::from_str(&self.identifier).unwrap()),
        }
    }
}

/// Saved data for an ip located in the cache.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PkgCache {
    units: Vec<UnitCache>,
}

impl PkgCache {
    pub fn new() -> Self {
        Self { units: Vec::new() }
    }
    /// Creates all the desired cached contents to be stored alongside an installed ip.
    pub fn from_ip(ip: &Ip) -> Result<Self, Fault> {
        // generate the unit map
        let umap = ip.collect_units(false, false, false)?;

        // build using an empty catalog because we only care about local internal links among design units for caching data
        let ip_graph = algo::compute_final_ip_graph(&ip, None, false)?;
        let files = algo::build_ip_file_list(&ip_graph, &ip);
        let global_graph = Plan::build_full_graph(&files)?;

        let mut units = Vec::new();

        for f in global_graph.get_map().iter() {
            // get the name of the design units that must come before this design unit
            let deps: Vec<&LangIdentifier> = global_graph
                .get_graph()
                .predecessors(global_graph.get_node_by_key(f.0).unwrap().index())
                .into_iter()
                .filter_map(|f| global_graph.get_key_by_index(f))
                .map(|n| n.get_suffix())
                .collect();
            let name = f.0.get_suffix().clone();
            let lunit = umap.get(&name).unwrap();
            units.push(UnitCache::from_graph_entry(
                &ip,
                name,
                f.1.as_ref(),
                lunit,
                deps,
            ));
        }

        Ok(Self { units: units })
    }

    /// Returns the file paths that belong to protected design units.
    pub fn get_protected(&self) -> Vec<String> {
        let mut protected = Vec::new();
        self.units
            .iter()
            .filter(|p| p.visibility.is_protected())
            .for_each(|f| protected.append(&mut f.sources.clone()));
        protected
    }

    pub fn get_units_mut(&mut self) -> &mut Vec<UnitCache> {
        &mut self.units
    }

    pub fn get_units(&self) -> &Vec<UnitCache> {
        &self.units
    }
}
