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

use crate::core::fileset;
use crate::util::anyerror::AnyError;
use cliproc::cli::Error;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::Write;
use std::{fs::File, path::PathBuf, str::FromStr};

use super::algo::ProjectFileNode;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Scheme {
    #[serde(rename = "tsv")]
    Tsv,
    #[serde(rename = "json")]
    Json,
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Tsv
    }
}

impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Tsv => "tsv",
                Self::Json => "json",
            }
        )
    }
}

impl FromStr for Scheme {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "tsv" => Ok(Self::Tsv),
            "json" => Ok(Self::Json),
            _ => Err(AnyError(format!("unknown file format: {}", s))),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct JsonEntry {
    fileset: String,
    library: String,
    filepath: String,
    dependencies: Vec<String>,
    /// Present only when some project in the graph declares a std
    #[serde(skip_serializing_if = "Option::is_none", default)]
    std: Option<String>,
}

/// Built-in fileset name for VHDL source code files
const VHDL: &str = "VHDL";
/// Built-in fileset name for Verilog source code files
const VLOG: &str = "VLOG";
/// Built-in fileset name for SystemVerilog source code files
const SYSV: &str = "SYSV";

#[derive(Debug, PartialEq)]
pub enum Entry<'a, 'b> {
    Hdl(&'b ProjectFileNode<'a>),
    Auxiliary(String, String, String),
}

impl<'a, 'b> Entry<'a, 'b> {
    fn get_builtin_fileset(node: &'b ProjectFileNode<'a>) -> &'b str {
        if fileset::is_verilog(node.get_file()) == true {
            VLOG
        } else if fileset::is_vhdl(node.get_file()) == true {
            VHDL
        } else if fileset::is_systemverilog(node.get_file()) == true {
            SYSV
        } else {
            panic!("unknown file in source file set")
        }
    }

    /// The owning project's declared language standard, or an empty
    /// string when the project declares none.
    fn node_std(node: &ProjectFileNode) -> String {
        node.get_project()
            .get_man()
            .get_project()
            .get_std()
            .clone()
            .unwrap_or_default()
    }

    pub fn to_string(&self, format: &Scheme, with_std: bool) -> String {
        match &format {
            Scheme::Tsv => match &self {
                Self::Hdl(node) => {
                    // match on what type of file we have
                    let source_set = Self::get_builtin_fileset(node);
                    if with_std {
                        format!(
                            "{}\t{}\t{}\t{}",
                            source_set,
                            node.get_library(),
                            node.get_file(),
                            Self::node_std(node)
                        )
                    } else {
                        format!(
                            "{}\t{}\t{}",
                            source_set,
                            node.get_library(),
                            node.get_file()
                        )
                    }
                }
                Self::Auxiliary(key, lib, file) => {
                    if with_std {
                        format!("{}\t{}\t{}\t", key, lib, file)
                    } else {
                        format!("{}\t{}\t{}", key, lib, file)
                    }
                }
            },
            Scheme::Json => serde_json::to_string_pretty(&self.to_json_entry(with_std)).unwrap(),
        }
    }

    fn to_json_entry(&self, with_std: bool) -> JsonEntry {
        match &self {
            Self::Hdl(node) => {
                let source_set = Self::get_builtin_fileset(node);
                JsonEntry {
                    fileset: source_set.to_string(),
                    library: node.get_library().to_string(),
                    filepath: node.get_file().to_string(),
                    dependencies: node.get_dep_files().clone(),
                    std: match with_std {
                        true => Some(Self::node_std(node)),
                        false => None,
                    },
                }
            }
            Self::Auxiliary(key, lib, file) => JsonEntry {
                fileset: key.to_string(),
                library: lib.to_string(),
                filepath: file.to_string(),
                dependencies: vec![],
                std: match with_std {
                    true => Some(String::new()),
                    false => None,
                },
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Blueprint<'a, 'b> {
    scheme: Scheme,
    steps: Vec<Entry<'a, 'b>>,
    /// When any project in the graph declares a `std`, every row carries
    /// its owning project's std as a trailing field; with no declared std
    /// anywhere the blueprint stays byte-compatible with the 3-field format.
    with_std: bool,
}

impl<'a, 'b> Default for Blueprint<'a, 'b> {
    fn default() -> Self {
        Self {
            scheme: Scheme::default(),
            steps: Vec::default(),
            with_std: false,
        }
    }
}

impl<'a, 'b> Blueprint<'a, 'b> {
    pub fn new(scheme: Scheme) -> Self {
        Self {
            scheme: scheme,
            steps: Vec::new(),
            with_std: false,
        }
    }

    /// Marks the blueprint to carry the trailing std field on every row.
    pub fn set_with_std(&mut self) {
        self.with_std = true;
    }

    pub fn get_filename(&self) -> String {
        String::from(match self.scheme {
            Scheme::Tsv => "blueprint.tsv",
            Scheme::Json => "blueprint.json",
        })
    }

    pub fn get_plan(&self) -> &Scheme {
        &self.scheme
    }

    /// Add the next instruction `instr` to the blueprint.
    pub fn add(&mut self, instr: Entry<'a, 'b>) {
        self.steps.push(instr);
    }

    pub fn write(&self, output_path: &PathBuf) -> Result<(PathBuf, usize), Error> {
        let blueprint_path = output_path.join(self.get_filename());
        let mut fd = File::create(&blueprint_path).expect("could not create blueprint file");
        // write the data.
        let data = self.to_string();
        fd.write_all(data.as_bytes())
            .expect("failed to write data to blueprint");
        Ok((blueprint_path, self.steps.len()))
    }
}

impl<'a, 'b> Display for Blueprint<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.scheme {
                Scheme::Tsv => self.steps.iter().fold(String::new(), |mut acc, i| {
                    acc.push_str(i.to_string(&self.scheme, self.with_std).as_ref());
                    acc.push('\n');
                    acc
                }),
                Scheme::Json => {
                    let entries: Vec<JsonEntry> = self
                        .steps
                        .iter()
                        .map(|m| m.to_json_entry(self.with_std))
                        .collect();
                    // add a new line because `to_string_pretty` forgets to :)
                    serde_json::to_string_pretty(&entries).unwrap() + "\n"
                }
            }
        )
    }
}
