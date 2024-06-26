use crate::util::anyerror::AnyError;
use cliproc::cli::Error;
use std::io::Write;
use std::{fs::File, path::PathBuf, str::FromStr};

use super::algo::IpFileNode;

#[derive(Debug, PartialEq)]
pub enum Scheme {
    Tsv,
    Json,
}

impl Default for Scheme {
    fn default() -> Self {
        Self::Tsv
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

#[derive(Debug, PartialEq)]
pub enum Instruction<'a, 'b> {
    Hdl(&'b IpFileNode<'a>),
    Supportive(String, String),
}

impl<'a, 'b> Instruction<'a, 'b> {
    pub fn write(&self, format: &Scheme) -> String {
        match &format {
            Scheme::Tsv => match &self {
                Self::Hdl(node) => format!(
                    "VHDL\t{}\t{}",
                    node.get_ip().get_man().get_hdl_library(),
                    node.get_file()
                ),
                Self::Supportive(key, file) => format!("{}\twork\t{}", key, file),
            },
            Scheme::Json => {
                todo!()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Blueprint<'a, 'b> {
    scheme: Scheme,
    steps: Vec<Instruction<'a, 'b>>,
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

    /// Add the next instruction `instr` to the blueprint.
    pub fn add(&mut self, instr: Instruction<'a, 'b>) {
        self.steps.push(instr);
    }

    pub fn write(&self, output_path: &PathBuf) -> Result<(PathBuf, usize), Error> {
        let blueprint_path = output_path.join(self.get_filename());
        let mut fd = File::create(&blueprint_path).expect("could not create blueprint file");
        // write the data
        let data = self.steps.iter().fold(String::new(), |mut acc, i| {
            acc.push_str(i.write(&self.scheme).as_ref());
            acc.push('\n');
            acc
        });
        fd.write_all(data.as_bytes())
            .expect("failed to write data to blueprint");
        Ok((blueprint_path, self.steps.len()))
    }
}
