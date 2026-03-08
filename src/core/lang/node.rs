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

use crate::core::algo::ProjectFileNode;
use crate::core::lang::vhdl::subunit::SubUnit;
use crate::core::lang::vhdl::symbols::{Context, VhdlSymbol};
use crate::util::anyerror::AnyError;
use colored::Colorize;

use super::reference::RefSet;
use super::sv::symbols::SystemVerilogSymbol;
use super::verilog::symbols::VerilogSymbol;
use super::verilog::symbols::module::Module;
use super::{Lang, LangIdentifier, VhdlIdentifier};
use crate::core::lang::vhdl::symbols::entity::Entity;

#[derive(Debug, PartialEq)]
pub enum HdlSymbol {
    Verilog(VerilogSymbol),
    Vhdl(VhdlSymbol),
    SystemVerilog(SystemVerilogSymbol),
    BlackBox(String),
}

impl HdlSymbol {
    pub fn get_name(&self) -> LangIdentifier {
        match &self {
            Self::Verilog(v) => LangIdentifier::Verilog(v.as_name().unwrap().clone()),
            Self::SystemVerilog(v) => LangIdentifier::SystemVerilog(v.as_name().unwrap().clone()),
            Self::Vhdl(v) => LangIdentifier::Vhdl(v.get_name().unwrap().clone()),
            Self::BlackBox(s) => LangIdentifier::Vhdl(VhdlIdentifier::Basic(s.to_string())),
        }
    }

    /// Checks if this symbol is a component/module/entity.
    pub fn is_component(&self) -> bool {
        match &self {
            Self::Verilog(v) => v.as_module().is_some(),
            Self::Vhdl(v) => v.as_entity().is_some(),
            Self::SystemVerilog(v) => v.as_module().is_some(),
            Self::BlackBox(_) => true,
        }
    }

    /// Checks if the symbol is a unit that can be shown in the hierarchy (Kind::All).
    pub fn is_hierarchical(&self) -> bool {
        match &self {
            Self::Verilog(_) => true,
            Self::Vhdl(v) => {
                v.as_entity().is_some() || v.as_context().is_some() || v.as_package().is_some()
            }
            Self::SystemVerilog(_) => true,
            Self::BlackBox(_) => true,
        }
    }

    pub fn get_refs(&self) -> Option<&RefSet> {
        match &self {
            Self::Verilog(v) => Some(v.get_refs()),
            Self::Vhdl(v) => Some(v.get_refs()),
            Self::SystemVerilog(v) => Some(v.get_refs()),
            Self::BlackBox(_) => None,
        }
    }

    /// Returns a cloned version of the ref set for the given HDL symbol.
    pub fn copy_refs(&self) -> Option<RefSet> {
        match &self {
            Self::Verilog(v) => Some(v.get_refs().clone()),
            Self::Vhdl(v) => Some(v.get_refs().clone()),
            Self::SystemVerilog(v) => Some(v.get_refs().clone()),
            Self::BlackBox(_) => None,
        }
    }

    /// Adds a batch of references, most commonly from a sub unit.
    pub fn inherit_refs(&mut self, refs: RefSet) {
        match self {
            Self::Vhdl(v) => {
                v.steal_refs(refs);
                ()
            }
            _ => (),
        }
    }

    /// Return the symbol as its [Module], if it is one.
    ///
    /// Works only on Verilog and SystemVerilog symbols.
    pub fn as_module(&self) -> Option<&Module> {
        match &self {
            Self::Verilog(v) => v.as_module(),
            Self::SystemVerilog(v) => v.as_module(),
            _ => None,
        }
    }

    /// Return the symbol as its [Entity], if it is one.
    ///
    /// Works only on VHDL symbols.
    pub fn as_entity(&self) -> Option<&Entity> {
        match &self {
            Self::Vhdl(v) => v.as_entity(),
            _ => None,
        }
    }

    /// Return the symbol as its [Context], if it is one.
    pub fn as_context(&self) -> Option<&Context> {
        match &self {
            Self::Vhdl(v) => v.as_context(),
            _ => None,
        }
    }

    pub fn as_entity_mut(&mut self) -> Option<&mut Entity> {
        match self {
            Self::Vhdl(v) => v.as_entity_mut(),
            _ => None,
        }
    }

    pub fn is_testbench(&self) -> bool {
        match &self {
            Self::Verilog(v) => {
                if let Some(m) = v.as_module() {
                    m.is_testbench()
                } else {
                    false
                }
            }
            Self::SystemVerilog(v) => {
                if let Some(m) = v.as_module() {
                    m.is_testbench()
                } else {
                    false
                }
            }
            Self::Vhdl(v) => {
                if let Some(e) = v.as_entity() {
                    e.is_testbench()
                } else {
                    false
                }
            }
            Self::BlackBox(_) => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HdlNode<'a> {
    sym: HdlSymbol,
    files: Vec<&'a ProjectFileNode<'a>>, // must use a vector to retain file order in blueprint
}

impl<'a> HdlNode<'a> {
    pub fn new(sym: HdlSymbol, file: &'a ProjectFileNode) -> Self {
        let mut set = Vec::with_capacity(1);
        set.push(file);
        Self {
            sym: sym,
            files: set,
        }
    }

    pub fn get_lang(&self) -> Lang {
        match self.sym {
            HdlSymbol::Verilog(_) => Lang::Verilog,
            HdlSymbol::Vhdl(_) => Lang::Vhdl,
            HdlSymbol::SystemVerilog(_) => Lang::SystemVerilog,
            HdlSymbol::BlackBox(_) => Lang::Vhdl,
        }
    }

    pub fn add_file(&mut self, ipf: &'a ProjectFileNode<'a>) {
        if self.files.contains(&ipf) == false {
            self.files.push(ipf);
        }
    }

    pub fn get_library(&self) -> LangIdentifier {
        self.files.get(0).as_ref().unwrap().get_library()
    }

    /// References the VHDL symbol
    pub fn get_symbol(&self) -> &HdlSymbol {
        &self.sym
    }

    pub fn get_symbol_mut(&mut self) -> &mut HdlSymbol {
        &mut self.sym
    }

    pub fn get_associated_files(&self) -> &Vec<&'a ProjectFileNode<'a>> {
        &self.files
    }

    pub fn is_black_box(&self) -> bool {
        self.files.is_empty()
    }

    pub fn black_box(sym: HdlSymbol) -> Self {
        Self {
            sym: sym,
            files: Vec::new(),
        }
    }

    pub fn display(&self, fmt: &IdentifierFormat) -> String {
        let name = self.sym.get_name().to_string();
        if self.is_black_box() == true {
            format!("{} {}", &name.yellow(), "?".yellow())
        } else {
            match fmt {
                IdentifierFormat::Long => {
                    let project = self.files.first().unwrap().get_project();
                    format!(
                        "{} ({})",
                        &name,
                        project.get_man().get_project().into_project_id_spec()
                    )
                }
                IdentifierFormat::Short => format!("{}", &name),
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SubUnitNode<'a> {
    sub: SubUnit,
    file: &'a ProjectFileNode<'a>,
}

impl<'a> SubUnitNode<'a> {
    pub fn new(unit: SubUnit, file: &'a ProjectFileNode<'a>) -> Self {
        Self {
            sub: unit,
            file: file,
        }
    }

    /// References the architecture struct.
    pub fn get_sub(&self) -> &SubUnit {
        &self.sub
    }

    /// References the project file node.
    pub fn get_file(&self) -> &'a ProjectFileNode<'a> {
        &self.file
    }
}

#[derive(Debug, PartialEq)]
pub enum IdentifierFormat {
    Long,
    Short,
}

impl std::str::FromStr for IdentifierFormat {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(AnyError(format!("format can be 'long' or 'short'"))),
        }
    }
}
