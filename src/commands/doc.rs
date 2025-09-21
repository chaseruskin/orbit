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

use std::path::PathBuf;

use crate::commands::helps::doc;
use crate::core::context::Context;
use crate::core::fileset::is_systemverilog;
use crate::core::fileset::is_verilog;
use crate::core::fileset::is_vhdl;
use crate::core::lang;
use crate::core::lang::sv::symbols::SystemVerilogSymbol;
use crate::core::lang::sv::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::vhdl::symbols::VhdlSymbol;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use crate::core::lang::LangIdentifier;
use crate::core::project::Project;
use crate::error::Error;
use crate::error::LastError;
use crate::info;
use crate::util::anyerror::Fault;
use crate::util::filesystem::LockZone;
use crate::warn;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Doc {
    target_dir: Option<String>,
}

impl Subcommand<Context> for Doc {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(doc::HELP))?;
        let command = Ok(Doc {
            target_dir: cli.get(Arg::option("target-dir").value("dir"))?,
        });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        println!("{}", "preparing to generate documentation...");
        // must be ran from the local project

        // verify running from a project directory and enter project's root directory
        c.jump_to_working_project()?;
        let current_project =
            Project::load(c.get_project_path().unwrap().to_path_buf(), true, false)?;

        let target_name = "doc";

        // get the output path where we will generate the documentation
        let default_target_dir = c.get_target_dir();
        let target_dir = self.target_dir.as_ref().unwrap_or(&default_target_dir);
        let out_dir = target_name;

        // path where all targets are to be kept
        let target_path = current_project.get_root().join(&target_dir);
        // path where the documentation will be kept
        let output_path = current_project.get_root().join(&target_dir).join(out_dir);

        // try to acquire a lock to only allow one orbit process access to the target output directory
        let (lockpath, _lockfd) = crate::util::filesystem::acquire_lock(
            &target_path,
            LockZone::OutputDir,
            Some(&target_name),
            false,
        )?;

        // outputs to a target/doc folder
        let result = self.run(&current_project, &output_path);

        // unlock the target output directory
        match std::fs::remove_file(&lockpath) {
            Ok(_) => (),
            Err(e) => {
                warn!(
                    "{}",
                    Error::FileUnlockFailed(lockpath.clone(), e.to_string(),).to_string()
                );
            }
        }
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::DocGenFailed(LastError(e.to_string())))?,
        }
    }
}

impl Doc {
    fn run(&self, p: &Project, output_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "generating documentation...");
        Self::document_project(p)?;
        info!(
            "documentation generated at: {}",
            crate::util::filesystem::into_std_str(output_path.to_path_buf())
        );
        Ok(())
    }
}

struct DocProject {}

impl DocProject {
    /// Writes the project documentation to files.
    pub fn save(&self, _path: &PathBuf) -> Result<(), Error> {
        todo!()
    }
}

impl Doc {
    /// Takes in a project and produces the project's documentation.
    fn document_project(prj: &Project) -> Result<DocProject, Fault> {
        // let units = prj.collect_units(false, false, false)?;

        let src_files = prj.gather_current_files();

        let mut all_doc_units = Vec::new();

        for src in &src_files {
            println!("documenting: {}", src);
            if is_vhdl(src) {
                all_doc_units.append(&mut Self::document_vhdl(src)?);
            } else if is_verilog(src) {
                all_doc_units.append(&mut Self::document_verilog(src)?);
            } else if is_systemverilog(src) {
                all_doc_units.append(&mut Self::document_sv(src)?);
            }
        }

        println!("{:#?}", all_doc_units);

        todo!()
    }

    fn document_verilog(src: &str) -> Result<Vec<DocUnit>, Fault> {
        Self::document_sv(src)
    }

    fn document_sv(src: &str) -> Result<Vec<DocUnit>, Fault> {
        // 1. create an in-order list of the statements and if they have a doc comment
        let mut doc_pairs: Vec<(Option<String>, SvStatement)> = Vec::new();

        let contents = lang::read_to_string(&src)?;
        let mut tokens = SystemVerilogTokenizer::from_source_code(&contents)
            .into_tokens_all()
            .into_iter()
            .peekable();

        let mut last_line = 0;
        let mut doc_comment = String::new();
        while let Some(t) = tokens.peek() {
            // if the next token is more than one line away, erase the doc comment
            if t.locate().line() > last_line + 1 {
                doc_comment = String::new();
            }
            if let Some(com) = t.as_ref().as_comment() {
                if let Some(dc) = com.into_doc_comment() {
                    doc_comment.push_str(&dc);
                    // record the line this comment was found at
                    last_line = t.locate().line();
                    tokens.next();
                    continue;
                }
            }
            let stmt = SystemVerilogSymbol::parse_doc_statement(&mut tokens)?;

            // handle distributing the comments to a statement
            if doc_comment.len() > 0 {
                doc_pairs.push((Some(doc_comment.clone()), stmt));
            } else {
                doc_pairs.push((None, stmt));
            }

            doc_comment = String::new();
        }
        // println!("{:#?}", doc_pairs);

        // 2. scan through the doc statement pairs and build doc structures
        let mut doc_pairs_iter = doc_pairs.into_iter();
        let mut doc_units = Vec::new();

        let mut cur_doc_unit = None;
        while let Some((doc_comment, stmt)) = doc_pairs_iter.next() {
            if let Some(name) = stmt.is_module_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                cur_doc_unit = Some(DocUnit::from_sv_stmt(
                    name,
                    None,
                    doc_comment,
                    stmt,
                    DocType::Module,
                ));
            } else if let Some(name) = stmt.is_package_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                cur_doc_unit = Some(DocUnit::from_sv_stmt(
                    name,
                    None,
                    doc_comment,
                    stmt,
                    DocType::Package,
                ));
            } else if let Some(cdu) = &mut cur_doc_unit {
                // try to update state
                cdu.update_sv_state(&stmt);
                // check if this statement if a valid thing to document for the current document item
                cdu.try_add_sv_doc_item(doc_comment, stmt);
            }
        }

        // push the last built doc unit to the list of doc units
        if let Some(du) = cur_doc_unit.take() {
            doc_units.push(du);
        }

        // 3. return doc structures
        Ok(doc_units)
    }

    fn document_vhdl(src: &str) -> Result<Vec<DocUnit>, Fault> {
        // 1. create an in-order list of the statements and if they have a doc comment
        let mut doc_pairs: Vec<(Option<String>, VhdlStatement)> = Vec::new();

        let contents = lang::read_to_string(&src)?;
        let mut tokens = VhdlTokenizer::from_source_code(&contents)
            .into_tokens_all()
            .into_iter()
            .peekable();

        let mut last_line = 0;
        let mut doc_comment = String::new();
        while let Some(t) = tokens.peek() {
            // if the next token is more than one line away, erase the doc comment
            if t.locate().line() > last_line + 1 {
                doc_comment = String::new();
            }
            if let Some(com) = t.as_ref().as_comment() {
                if let Some(dc) = com.into_doc_comment() {
                    doc_comment.push_str(&dc);
                    // record the line this comment was found at
                    last_line = t.locate().line();
                    tokens.next();
                    continue;
                }
            }
            let stmt = VhdlSymbol::parse_doc_statement(&mut tokens);

            // handle distributing the comments to a statement
            if doc_comment.len() > 0 {
                doc_pairs.push((Some(doc_comment.clone()), stmt));
            } else {
                doc_pairs.push((None, stmt));
            }

            doc_comment = String::new();
        }

        // 2. scan through the doc statement pairs and build doc structures
        let mut doc_pairs_iter = doc_pairs.into_iter();
        let mut doc_units = Vec::new();

        let mut cur_doc_unit = None;
        while let Some((doc_comment, stmt)) = doc_pairs_iter.next() {
            if let Some(name) = stmt.is_entity_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                cur_doc_unit = Some(DocUnit::from_vhdl_stmt(
                    name,
                    None,
                    doc_comment,
                    stmt,
                    DocType::Entity,
                ));
            } else if let Some((arch_name, ent_name)) = stmt.is_arch_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                cur_doc_unit = Some(DocUnit::from_vhdl_stmt(
                    arch_name,
                    Some(ent_name),
                    doc_comment,
                    stmt,
                    DocType::Architecture,
                ));
            } else if let Some(name) = stmt.is_package_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                cur_doc_unit = Some(DocUnit::from_vhdl_stmt(
                    name,
                    None,
                    doc_comment,
                    stmt,
                    DocType::Package,
                ));
            } else if stmt.is_package_body_decl() {
                // push the previously built doc unit to the list of doc units
                if let Some(du) = cur_doc_unit.take() {
                    doc_units.push(du);
                }
                // NOTE: do not build a document unit for the package body

                // handle adding statements to the current doc unit (if it exists)
            } else if let Some(cdu) = &mut cur_doc_unit {
                // try to update state
                cdu.update_vhdl_state(&stmt);
                // check if this statement if a valid thing to document for the current document item
                cdu.try_add_vhdl_doc_item(doc_comment, stmt);
            }
        }

        // push the last built doc unit to the list of doc units
        if let Some(du) = cur_doc_unit.take() {
            doc_units.push(du);
        }

        // 3. return doc structures
        Ok(doc_units)
    }
}

use crate::core::lang::sv::symbols::Statement as SvStatement;
use crate::core::lang::sv::symbols::Statement as VerilogStatement;
use crate::core::lang::vhdl::symbols::Statement as VhdlStatement;

#[derive(Debug, PartialEq, Clone, Copy)]
enum DocType {
    None,
    Entity,
    Package,
    Architecture,
    Generic,
    Port,
    Function,
    Task,
    Procedure,
    Type,
    Subtype,
    Module,
    Struct,
    Enum,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum DocState {
    Norm,
    Generic,
    Port,
}

#[derive(Debug, PartialEq)]
struct DocItem {
    pub name: Option<LangIdentifier>,
    pub parent: Option<LangIdentifier>,
    pub form: DocType,
    pub docs: Option<String>,
    pub stmt: Statement,
}

#[derive(Debug, PartialEq)]
enum Statement {
    Vhdl(VhdlStatement),
    Verilog(VerilogStatement),
    SystemVerilog(SvStatement),
}

impl Statement {
    pub fn new() -> Self {
        Self::Vhdl(VhdlStatement::new())
    }
}

impl DocItem {
    pub fn new() -> Self {
        Self {
            name: None,
            parent: None,
            form: DocType::None,
            docs: None,
            stmt: Statement::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DocUnit {
    // state is used to help internally track/identify certain statements
    state: DocState,
    unit: DocItem,
    items: Vec<DocItem>,
}

impl DocUnit {
    pub fn new() -> Self {
        Self {
            state: DocState::Norm,
            unit: DocItem::new(),
            items: Vec::new(),
        }
    }

    pub fn get_name(&self) -> Option<&LangIdentifier> {
        self.unit.name.as_ref()
    }

    pub fn as_parent_name(&self) -> Option<&LangIdentifier> {
        self.unit.parent.as_ref()
    }
}

// VHDL-specific functions
impl DocUnit {
    pub fn from_vhdl_stmt(
        name: LangIdentifier,
        parent: Option<LangIdentifier>,
        cmt: Option<String>,
        stmt: VhdlStatement,
        form: DocType,
    ) -> Self {
        Self {
            state: DocState::Norm,
            unit: DocItem {
                name: Some(name),
                parent: parent,
                form: form,
                docs: cmt,
                stmt: Statement::Vhdl(stmt),
            },
            items: Vec::new(),
        }
    }

    /// Attempts to update the doc item's internal state if needed.
    pub fn update_vhdl_state(&mut self, stmt: &VhdlStatement) {
        use crate::core::lang::vhdl::token::keyword::Keyword;
        match self.unit.form {
            DocType::Entity => {
                if stmt.starts_with_kw(Keyword::Generic) {
                    self.state = DocState::Generic;
                } else if stmt.starts_with_kw(Keyword::Port) {
                    self.state = DocState::Port;
                } else if stmt.starts_with_kw(Keyword::Begin) {
                    self.state = DocState::Norm;
                }
            }
            _ => (),
        }
    }

    /// Tries to add the doc item to the list of documentation items for this unit, if the doc unit is in
    /// an acceptable state and the statement is valid.
    pub fn try_add_vhdl_doc_item(&mut self, cmt: Option<String>, stmt: VhdlStatement) {
        match self.unit.form {
            DocType::Entity => {
                if stmt.can_be_interface()
                    && (self.state == DocState::Generic || self.state == DocState::Port)
                {
                    if self.state == DocState::Generic {
                        self.items.push(DocItem {
                            name: stmt.get_first_identifier(),
                            parent: None,
                            form: DocType::Generic,
                            docs: cmt,
                            stmt: Statement::Vhdl(stmt),
                        });
                    } else {
                        self.items.push(DocItem {
                            name: stmt.get_first_identifier(),
                            parent: None,
                            form: DocType::Port,
                            docs: cmt,
                            stmt: Statement::Vhdl(stmt),
                        });
                    }
                }
            }
            DocType::Package => {
                if stmt.is_function() {
                    self.items.push(DocItem {
                        name: stmt.get_first_identifier(),
                        parent: None,
                        form: DocType::Function,
                        docs: cmt,
                        stmt: Statement::Vhdl(stmt),
                    });
                } else if stmt.is_type() {
                    self.items.push(DocItem {
                        name: stmt.get_first_identifier(),
                        parent: None,
                        form: DocType::Type,
                        docs: cmt,
                        stmt: Statement::Vhdl(stmt),
                    });
                } else if stmt.is_subtype() {
                    self.items.push(DocItem {
                        name: stmt.get_first_identifier(),
                        parent: None,
                        form: DocType::Subtype,
                        docs: cmt,
                        stmt: Statement::Vhdl(stmt),
                    });
                }
            }
            _ => (),
        }
    }
}

// Verilog/SV-specific functions
impl DocUnit {
    pub fn from_sv_stmt(
        name: LangIdentifier,
        parent: Option<LangIdentifier>,
        cmt: Option<String>,
        stmt: SvStatement,
        form: DocType,
    ) -> Self {
        Self {
            state: DocState::Port,
            unit: DocItem {
                name: Some(name),
                parent: parent,
                form: form,
                docs: cmt,
                stmt: Statement::SystemVerilog(stmt),
            },
            items: Vec::new(),
        }
    }

    /// Attempts to update the doc item's internal state if needed.
    pub fn update_sv_state(&mut self, stmt: &SvStatement) {
        match self.unit.form {
            DocType::Module => {
                if stmt.is_param_stmt() {
                    self.state = DocState::Generic;
                } else if stmt.is_port_stmt() {
                    self.state = DocState::Port;
                } else if stmt.is_end_mod_decl() {
                    self.state = DocState::Norm;
                }
            }
            _ => (),
        }
    }

    /// Tries to add the doc item to the list of documentation items for this unit, if the doc unit is in
    /// an acceptable state and the statement is valid.
    pub fn try_add_sv_doc_item(&mut self, cmt: Option<String>, stmt: SvStatement) {
        use crate::core::lang::sv::token::keyword::Keyword;
        match self.unit.form {
            DocType::Module => {
                if stmt.can_be_interface()
                    && (self.state == DocState::Generic || self.state == DocState::Port)
                {
                    if self.state == DocState::Generic {
                        self.items.push(DocItem {
                            name: None,
                            parent: None,
                            form: DocType::Generic,
                            docs: cmt,
                            stmt: Statement::SystemVerilog(stmt),
                        });
                    } else {
                        self.items.push(DocItem {
                            name: None,
                            parent: None,
                            form: DocType::Port,
                            docs: cmt,
                            stmt: Statement::SystemVerilog(stmt),
                        });
                    }
                }
            }
            DocType::Package => {
                // determine what this statement is describing within the package
                let form = if stmt.starts_with_kw(Keyword::Task) {
                    DocType::Task
                } else if stmt.starts_with_kw(Keyword::Function) {
                    DocType::Function
                } else if stmt.is_typedef_struct() {
                    DocType::Struct
                } else if stmt.is_typedef_enum() {
                    DocType::Enum
                } else {
                    DocType::None
                };

                if form != DocType::None {
                    self.items.push(DocItem {
                        name: None,
                        parent: None,
                        form: form,
                        docs: cmt,
                        stmt: Statement::SystemVerilog(stmt),
                    });
                }
            }
            _ => (),
        }
    }
}
