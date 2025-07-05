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

    pub fn to_string(&self, format: &Scheme) -> String {
        match &format {
            Scheme::Tsv => match &self {
                Self::Hdl(node) => {
                    // match on what type of file we have
                    let source_set = Self::get_builtin_fileset(node);
                    format!(
                        "{}\t{}\t{}",
                        source_set,
                        node.get_library(),
                        node.get_file()
                    )
                }
                Self::Auxiliary(key, lib, file) => format!("{}\t{}\t{}", key, lib, file),
            },
            Scheme::Json => serde_json::to_string_pretty(&self.to_json_entry()).unwrap(),
        }
    }

    fn to_json_entry(&self) -> JsonEntry {
        match &self {
            Self::Hdl(node) => {
                let source_set = Self::get_builtin_fileset(node);
                JsonEntry {
                    fileset: source_set.to_string(),
                    library: node.get_library().to_string(),
                    filepath: node.get_file().to_string(),
                    dependencies: node.get_dep_files().clone(),
                }
            }
            Self::Auxiliary(key, lib, file) => JsonEntry {
                fileset: key.to_string(),
                library: lib.to_string(),
                filepath: file.to_string(),
                dependencies: vec![],
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Blueprint<'a, 'b> {
    scheme: Scheme,
    steps: Vec<Entry<'a, 'b>>,
}

impl<'a, 'b> Default for Blueprint<'a, 'b> {
    fn default() -> Self {
        Self {
            scheme: Scheme::default(),
            steps: Vec::default(),
        }
    }
}

impl<'a, 'b> Blueprint<'a, 'b> {
    pub fn new(scheme: Scheme) -> Self {
        Self {
            scheme: scheme,
            steps: Vec::new(),
        }
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
        // write the data
        let data = match &self.scheme {
            Scheme::Tsv => self.steps.iter().fold(String::new(), |mut acc, i| {
                acc.push_str(i.to_string(&self.scheme).as_ref());
                acc.push('\n');
                acc
            }),
            Scheme::Json => {
                let entries: Vec<JsonEntry> =
                    self.steps.iter().map(|m| m.to_json_entry()).collect();
                // add a new line because `to_string_pretty` forgets to :)
                serde_json::to_string_pretty(&entries).unwrap() + "\n"
            }
        };
        fd.write_all(data.as_bytes())
            .expect("failed to write data to blueprint");
        Ok((blueprint_path, self.steps.len()))
    }
}
