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
use std::process::Command;

use crate::commands::helps::doc;
use crate::commands::plan;
use crate::core::catalog::Catalog;
use crate::core::context;
use crate::core::context::Context;
use crate::core::fileset::is_systemverilog;
use crate::core::fileset::is_verilog;
use crate::core::fileset::is_vhdl;
use crate::core::lang;
use crate::core::lang::sv::symbols::SystemVerilogSymbol;
use crate::core::lang::sv::token::keyword::Keyword as SvKeyword;
use crate::core::lang::sv::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::vhdl::symbols::VhdlSymbol;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use crate::core::lang::Lang;
use crate::core::lang::LangIdentifier;
use crate::core::lang::LangUnit;
use crate::core::lockfile::LockEntry;
use crate::core::lockfile::LockFile;
use crate::core::project::Project;
use crate::core::version::AnyVersion;
use crate::error::Error;
use crate::error::LastError;
use crate::info;
use crate::util::anyerror::Fault;
use crate::util::environment;
use crate::util::filesystem::LockZone;
use crate::util::filesystem::{PRJ_CATALOG_EX_LOCK_NAME, PRJ_CATALOG_SH_LOCK_NAME};
use crate::util::sha256;
use crate::warn;
use ansi_to_html::Converter;
use markdown;
use std::collections::HashMap;
use std::path::Path;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

type UnitMap = HashMap<LangIdentifier, LangUnit>;

#[derive(Debug, PartialEq)]
pub struct Doc {
    target_dir: Option<String>,
    doc_priv_items: bool,
    no_deps: bool,
    open_browser: bool,
    // IDEA: add option to select output format (HTML, MD)?
}

impl Subcommand<Context> for Doc {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(doc::HELP))?;
        let command = Ok(Doc {
            no_deps: cli.check(Arg::flag("no-deps"))?,
            open_browser: cli.check(Arg::flag("open"))?,
            doc_priv_items: cli.check(Arg::flag("document-private-units"))?,
            target_dir: cli.get(Arg::option("target-dir").value("dir"))?,
        });
        command
    }

    fn execute(self, c: &Context) -> proc::Result {
        // println!("{}", "preparing to generate documentation...");
        // must be ran from the local project

        // verify running from a project directory and enter project's root directory
        c.jump_to_working_project()?;
        let current_project =
            Project::load(c.get_project_path().unwrap().to_path_buf(), true, false)?;

        let target_name = "doc";

        // verify the browser env variable exists
        let browser = if self.open_browser == true {
            match std::env::var(environment::BROWSER) {
                Ok(browser) => Some(browser),
                Err(e) => match e {
                    std::env::VarError::NotPresent => {
                        return Err(Box::new(Error::BrowserEnvVarMissing))
                    }
                    _ => return Err(Box::new(e)),
                },
            }
        } else {
            None
        };

        // get the output path where we will generate the documentation
        let default_target_dir = c.get_target_dir();
        let target_dir = self.target_dir.as_ref().unwrap_or(&default_target_dir);
        let out_dir = target_name;

        // path where all targets are to be kept
        let target_path = current_project.get_root().join(&target_dir);
        // path where the documentation will be kept
        let output_path = current_project.get_root().join(&target_dir).join(out_dir);

        // before we gather the catalog, request an "APPEND" action to the cache
        let (_cache_ap_path, cache_ap_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_EX_LOCK_NAME),
            false,
        )?;

        // gather the catalog and resolve any missing dependencies
        let mut catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?
            .available(&c.get_config().get_channels())?;
        let updated_lf = plan::resolve_missing_deps(c, &current_project, &mut catalog, false)?;
        let catalog = catalog;

        // try to acquire a lock to only allow one orbit process access to the target output directory
        let (lockpath, _lockfd) = crate::util::filesystem::acquire_lock(
            &target_path,
            LockZone::OutputDir,
            Some(&target_name),
            false,
        )?;

        // before we read the source files in our process, request a shared "READ" action to the cache
        let (_cache_rd_path, cache_rd_lock) = crate::util::filesystem::acquire_lock(
            c.get_home_path(),
            LockZone::ProjectCatalog,
            Some(PRJ_CATALOG_SH_LOCK_NAME),
            true,
        )?;

        // release our "APPEND" action to the cache
        crate::util::filesystem::release_lock(&cache_ap_lock)?;

        // outputs to a target/doc folder
        crate::info!("executing target {}", target_name.green());
        let result = self.run(
            &current_project,
            &updated_lf,
            &catalog,
            &output_path,
            true,
            c.are_units_private_by_default(),
            browser,
        );

        // release our "READ" action to the cache
        crate::util::filesystem::release_lock(&cache_rd_lock)?;

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
    fn run(
        &self,
        prj: &Project,
        updated_lf: &Option<LockFile>,
        catalog: &Catalog,
        output_path: &PathBuf,
        clean: bool,
        priv_by_default: bool,
        browser: Option<String>,
    ) -> Result<(), Fault> {
        // check if to clean the target output directory (but keep the file lock!!)
        if clean == true && Path::exists(&output_path) == true {
            std::fs::remove_dir_all(&output_path)?;
        }

        // Pick the most up-to-date lockfile
        let lf = match updated_lf {
            Some(lf) => lf,
            None => prj.get_lock(),
        };

        let mut all_doc_prjs = Vec::new();
        // generate documentation for all dependencies
        for entry in lf.inner() {
            let prj_to_doc = match entry.matches_target(&LockEntry::from((prj, true))) {
                // use the local project
                true => Some(prj),
                false => {
                    // skip all other projects if no dependencies is specified as an option
                    if self.no_deps == true {
                        continue;
                    }
                    // identify if it is a relative path entry
                    match entry.is_relative() {
                        true => prj
                            .get_man()
                            .get_deps()
                            .get(entry.get_name())
                            .unwrap()
                            .as_project(),
                        false => {
                            let any_ver =
                                AnyVersion::Specific(entry.get_version().to_partial_version());
                            match catalog.inner().get(entry.get_uuid()) {
                                Some(stat) => stat.get_install(&any_ver),
                                None => None,
                            }
                        }
                    }
                }
            };
            // generate the documentation for this project
            if let Some(doc_prj) = prj_to_doc {
                // skip dynamic projects
                if doc_prj.is_dynamic() {
                    continue;
                }
                info!("documenting {}", entry.to_project_id_spec());
                let doc_prj = DocProject::from_project(&doc_prj)?;
                all_doc_prjs.push(doc_prj);
            }
        }

        let index_path = self.save(
            output_path,
            all_doc_prjs,
            self.doc_priv_items,
            priv_by_default,
        )?;

        info!(
            "documentation generated at: {:?}",
            crate::util::filesystem::into_std_str(index_path.clone())
        );

        if let Some(browser) = browser {
            let _ = Command::new(browser).arg(index_path).spawn()?;
        }

        Ok(())
    }
}

struct DocProject<'a> {
    doc_units: Vec<DocUnit>,
    project: &'a Project,
}

/// Converts Markdown syntax (GFM) into valid HTML.
///
/// See GFM specification: https://github.github.com/gfm/.
pub fn to_html(s: &str) -> String {
    inject_css_style(&markdown::to_html_with_options(s, &markdown::Options::gfm()).unwrap())
}

/// Accepts an HTML formatted string `s` and adds certain styling options to the HTML
/// using inline CSS.
fn inject_css_style(s: &str) -> String {
    let result = s.replace(
        "<table>",
        r#"<table style="width: 100%; border-collapse: collapse;">"#,
    );
    let result = result.replace(
        "<td ",
        r#"<td style="border: 1px solid black; padding: 8px;""#,
    );
    let result = result.replace(
        "<th ",
        r#"<th style="border: 1px solid black; padding: 8px;""#,
    );
    // Fix broken newlines
    let result = result.replace("&lt;br&gt;", "<br>");
    // apply ASNI to HTML color conversion
    let converter = Converter::new().skip_escape(true).skip_optimize(true);
    let result = converter.convert(&result).unwrap();
    result
}

impl<'a> DocProject<'a> {
    /// Generates the project's unique string for the directory where it shall store all of its contents.
    pub fn into_out_name(&self) -> String {
        let prj_tbl = self.project.get_man().get_project();
        format!(
            "{0}-{1}-{2}",
            prj_tbl.get_name(),
            prj_tbl.get_version(),
            prj_tbl.get_uuid()
        )
    }

    /// Writes the project documentation to files.
    ///
    /// Assumes `output_path` is the directory where to create the doc project's
    /// artifacts.
    pub fn save(
        &self,
        output_path: &PathBuf,
        doc_priv_items: bool,
        priv_by_default: bool,
    ) -> Result<(), Fault> {
        // use a unique name to avoid path conflicts of same project, multiple version
        let root_dir = output_path.join(self.into_out_name());
        // create the root directory
        std::fs::create_dir_all(&root_dir)?;
        // try to collect all design units (force to ensure we apply the correct private visibility capture)
        let prj_unit_map =
            self.project
                .collect_units(false, doc_priv_items == false, priv_by_default)?;
        // write the project's main index file
        self.write_index_file(&root_dir, &prj_unit_map, doc_priv_items)?;
        // write all source files
        self.write_source_files(&root_dir, &prj_unit_map)?;
        Ok(())
    }

    fn write_source_files(
        &self,
        output_path: &PathBuf,
        prj_unit_map: &HashMap<LangIdentifier, LangUnit>,
    ) -> Result<(), Fault> {
        let source_dus = self.doc_units.iter().filter(|f| {
            f.get_name().is_some()
                && (f.is_type(DocType::Entity)
                    | f.is_type(DocType::Package)
                    | f.is_type(DocType::Module))
        });

        let doc_subus: Vec<&DocUnit> = self
            .doc_units
            .iter()
            .filter(|f| f.get_name().is_some() && f.as_parent_name().is_some())
            .collect();

        // Write each source code file into the documentation
        for ds in &doc_subus {
            Self::import_hdl_source_to_md(output_path, &ds.get_source())?;
        }

        // Write each primary design unit as its own documentation page
        for du in source_dus {
            let unit_subus: Vec<&&DocUnit> = doc_subus
                .iter()
                .filter(|f| f.as_parent_name() == du.get_name())
                .collect();
            self.write_source_file(output_path, du, &prj_unit_map, unit_subus)?;
        }
        Ok(())
    }

    /// Returns the name of the file that will contain the raw HDL source code for the documentation generation.
    fn get_hdl_source_md_name(src_file: &str) -> (String, Lang) {
        let src_src_path = src_file;
        let lang = match is_systemverilog(src_src_path) {
            true => Lang::SystemVerilog,
            false => match is_vhdl(src_src_path) {
                true => Lang::Vhdl,
                false => Lang::Verilog,
            },
        };
        let sha = sha256::compute_sha256(src_src_path.as_bytes()).to_string_short();
        let ext = match lang {
            Lang::SystemVerilog => "sv",
            Lang::Vhdl => "vhd",
            Lang::Verilog => "v",
        };

        let src_path = std::path::PathBuf::from(src_src_path);
        let file_name = src_path.file_stem().unwrap_or_default().to_string_lossy();
        let md_src_file = format!("{}.{}.{}.html", file_name, ext, sha);
        (md_src_file, lang)
    }

    /// Writes the raw HDL source file to a dedicated document file such that the contents can be inspected in HTML in the
    /// browser.
    ///
    /// Returns the path to the destination source file.
    fn import_hdl_source_to_md(output_path: &PathBuf, src_file: &str) -> Result<String, Fault> {
        // copy the source file contents to a new markdown file
        let (md_src_file, lang) = Self::get_hdl_source_md_name(src_file);
        let tar_src_path = output_path.join(&md_src_file);
        // only write the contents if the file does not exist
        if tar_src_path.exists() == false {
            let md_lang = match lang {
                Lang::SystemVerilog => "sv",
                Lang::Vhdl => "vhdl",
                Lang::Verilog => "verilog",
            };
            let raw_src = lang::read_to_string(&src_file)?;
            let src_contents = format!("``` {}\n{}\n\n```\n", md_lang, raw_src);
            std::fs::write(tar_src_path, to_html(&src_contents))?;
        }
        Ok(md_src_file)
    }

    /// Writes the documentation page for the given design unit.
    fn write_source_file(
        &self,
        output_path: &PathBuf,
        du: &DocUnit,
        unit_map: &UnitMap,
        unit_subus: Vec<&&DocUnit>,
    ) -> Result<(), Fault> {
        let du_name = du.get_name().unwrap();
        let unit_path = output_path.join(&format!("{}.html", du_name));

        let mut contents = String::new();
        // try to find the source file
        let src_file = match unit_map.get(du_name) {
            Some(lu) => Some(Self::import_hdl_source_to_md(
                output_path,
                lu.get_source_file(),
            )?),
            None => None,
        };

        if let Some(lu) = unit_map.get(du_name) {
            contents.push_str(&to_html(&du.to_markdown(src_file, lu, unit_subus)));
        }

        std::fs::write(&unit_path, contents)?;
        Ok(())
    }

    /// Adds a section list for the project's overall index file.
    fn add_index_section(
        &self,
        section: &str,
        dtype: DocType,
        prj_unit_map: &HashMap<LangIdentifier, LangUnit>,
        doc_priv_items: bool,
    ) -> String {
        let mut contents = String::new();
        let filtered_dus = self.doc_units.iter().filter(|f| f.is_type(dtype));
        let mut said_title = false;
        for du in filtered_dus {
            if let Some(name) = du.get_name() {
                // Skip if the unit does not have the right visibility (and we don't want to show private items)
                if doc_priv_items == false
                    && (prj_unit_map.get(name).is_none()
                        || prj_unit_map
                            .get(name)
                            .unwrap()
                            .get_visibility()
                            .is_private())
                {
                    continue;
                }
                // Start with saying the title of the section
                if said_title == false {
                    contents.push_str(&format!("## {}\n", section));
                    said_title = true;
                }
                // try to extract out the unit's description
                let desc = match du.get_first_sentence() {
                    Some(sum) => format!(": {}", Doc::fix_sentence(&sum)),
                    None => String::new(),
                };
                contents.push_str(&format!("- [__{}__]({}.html){}\n", name, name, desc));
            }
        }
        if said_title == true {
            contents.push_str("\n");
        }
        contents
    }

    fn write_index_file(
        &self,
        output_path: &PathBuf,
        prj_unit_map: &HashMap<LangIdentifier, LangUnit>,
        doc_priv_items: bool,
    ) -> Result<(), Fault> {
        let index_path = output_path.join("index.html");
        let name = self.project.get_man().get_project().get_name().to_string();
        let desc = self
            .project
            .get_man()
            .get_project()
            .get_description()
            .clone()
            .unwrap_or(String::new());

        // fix up the descripton
        let mut contents = String::new();
        // add start to the document
        contents.push_str(&format!(
            "# Project {}\n{}\n\n",
            name,
            Doc::fix_sentence(&desc)
        ));

        // Add entity list
        contents.push_str(&self.add_index_section(
            "Entities",
            DocType::Entity,
            prj_unit_map,
            doc_priv_items,
        ));

        // Add module list
        contents.push_str(&self.add_index_section(
            "Modules",
            DocType::Module,
            prj_unit_map,
            doc_priv_items,
        ));

        // Add package list
        contents.push_str(&self.add_index_section(
            "Packages",
            DocType::Package,
            prj_unit_map,
            doc_priv_items,
        ));

        std::fs::write(&index_path, to_html(&contents))?;
        Ok(())
    }

    /// Takes in a project and produces the project's documentation.
    pub fn from_project(prj: &'a Project) -> Result<DocProject<'a>, Fault> {
        let doc_units = Doc::collect_doc_units(prj)?;
        Ok(Self {
            doc_units: doc_units,
            project: prj,
        })
    }
}

impl Doc {
    /// Applies proper ending punctuation and first letter capitalization to a string.
    fn fix_sentence(s: &str) -> String {
        let s = s.trim();
        if s.len() == 0 {
            return String::new();
        }
        // check if we need a period at the end
        let new_ending = match s.chars().last().unwrap_or_default() {
            '.' | '?' | '!' => "",
            _ => ".",
        };
        // verify we are starting with a capital letter
        let new_start = s
            .chars()
            .next()
            .unwrap_or_default()
            .to_uppercase()
            .to_string();

        format!(
            "{}{}{}",
            new_start,
            s.get(1..).unwrap_or_default().to_string(),
            new_ending
        )
    }

    /// Saves all the provided Documentation projects and returns the single entry INDEX file path.
    fn save(
        &self,
        output_path: &PathBuf,
        doc_projects: Vec<DocProject>,
        doc_priv_items: bool,
        priv_by_default: bool,
    ) -> Result<PathBuf, Fault> {
        std::fs::create_dir_all(&output_path)?;
        // create a cache tag file if does not exist
        match Context::is_cache_tag_valid(&output_path.parent().unwrap().to_path_buf()) {
            Ok(_) => (),
            Err(e) => std::fs::write(&e, context::CACHE_TAG)?,
        }
        // track the contents for the entry index file
        let index_path = output_path.join("index.html");
        let mut index_data = String::new();
        index_data.push_str(&format!("# Projects\n"));
        // save each project
        for dp in doc_projects {
            dp.save(output_path, doc_priv_items, priv_by_default)?;
            index_data.push_str(&format!(
                "- [__{0}__ __{1}__ ({2})]({4}/index.html){3}\n",
                dp.project.get_man().get_project().get_name(),
                dp.project.get_man().get_project().get_version(),
                dp.project.get_man().get_project().get_uuid(),
                {
                    if let Some(desc) = dp.project.get_man().get_project().get_description() {
                        format!(": {}", Doc::fix_sentence(&desc))
                    } else {
                        String::new()
                    }
                },
                dp.into_out_name(),
            ));
        }

        // write the entry index file
        std::fs::write(&index_path, to_html(&index_data))?;

        Ok(index_path)
    }

    fn collect_doc_units(prj: &Project) -> Result<Vec<DocUnit>, Fault> {
        let src_files = prj.gather_current_files();

        let mut all_doc_units = Vec::new();

        for src in &src_files {
            // println!("documenting: {}", src);
            if is_vhdl(src) {
                all_doc_units.append(&mut Self::document_vhdl(src)?);
            } else if is_verilog(src) {
                all_doc_units.append(&mut Self::document_verilog(src)?);
            } else if is_systemverilog(src) {
                all_doc_units.append(&mut Self::document_sv(src)?);
            }
        }

        // println!("{:#?}", all_doc_units);
        Ok(all_doc_units)
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
        let mut last_stmt_line = 0;
        let mut doc_comment = String::new();
        while let Some(t) = tokens.peek() {
            // if the next token is more than one line away, erase the doc comment
            if t.locate().line() > last_line + 1 {
                doc_comment = String::new();
            }
            if let Some(com) = t.as_ref().as_comment() {
                if let Some(dc) = com.into_doc_comment() {
                    // check if we are on the same line as the previous statement
                    if doc_pairs.len() > 0 && last_stmt_line == t.locate().line() {
                        let last_doc_pair = doc_pairs.pop().unwrap();
                        doc_pairs.push((Some(dc), last_doc_pair.1));
                        doc_comment = String::new();
                        last_line = t.locate().line();
                        tokens.next();
                        continue;
                    } else {
                        doc_comment.push_str(&dc);
                        // record the line this comment was found at
                        last_line = t.locate().line();
                        tokens.next();
                        continue;
                    }
                }
            }
            let stmt = SystemVerilogSymbol::parse_doc_statement(&mut tokens)?;
            last_stmt_line = stmt.get_ending_line_no();
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
                    src.to_string(),
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
                    src.to_string(),
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
        let mut last_stmt_line = 0;
        let mut doc_comment = String::new();
        while let Some(t) = tokens.peek() {
            // if the next token is more than one line away, erase the doc comment
            if t.locate().line() > last_line + 1 {
                doc_comment = String::new();
            }
            if let Some(com) = t.as_ref().as_comment() {
                if let Some(dc) = com.into_doc_comment() {
                    // check if we are on the same line as the previous statement
                    if doc_pairs.len() > 0 && last_stmt_line == t.locate().line() {
                        let last_doc_pair = doc_pairs.pop().unwrap();
                        doc_pairs.push((Some(dc), last_doc_pair.1));
                        doc_comment = String::new();
                        last_line = t.locate().line();
                        tokens.next();
                        continue;
                    } else {
                        doc_comment.push_str(&dc);
                        // record the line this comment was found at
                        last_line = t.locate().line();
                        tokens.next();
                        continue;
                    }
                }
            }
            let stmt = VhdlSymbol::parse_doc_statement(&mut tokens);
            last_stmt_line = stmt.get_ending_line_no();
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
                    src.to_string(),
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
                    src.to_string(),
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
                    src.to_string(),
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

    pub fn contains_identifier(&self, id: LangIdentifier) -> bool {
        match &self {
            Self::SystemVerilog(stmt) => stmt.has_matching_id(id.as_sv_name().unwrap()),
            Self::Verilog(stmt) => stmt.has_matching_id(id.as_verilog_name().unwrap()),
            Self::Vhdl(stmt) => stmt.has_matching_id(id.as_vhdl_name().unwrap()),
        }
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vhdl(stmt) => write!(f, "{}", stmt.to_colored_string()),
            Self::Verilog(stmt) => write!(f, "{}", stmt.to_colored_string()),
            Self::SystemVerilog(stmt) => write!(f, "{}", stmt.to_colored_string()),
        }
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

    /// Returns the relevant documentation for this item.
    pub fn get_docs(&self) -> Option<&String> {
        self.docs.as_ref()
    }

    /// Attempts to extract out the first sentence from the documentation.
    ///
    /// Returns none if cannot identify any text deemed as a single sentence.
    pub fn get_first_sentence(&self) -> Option<String> {
        if let Some(docs) = &self.docs {
            let mut sentences = docs.split(&['.', '?', '!']);
            if let Some(s) = sentences.next() {
                // try to find double newline
                let mut last_split = s.split("\n\n");
                if let Some(t) = last_split.next() {
                    return Some(t.trim_end().to_string());
                }
            }
        }
        None
    }

    pub fn get_name(&self) -> Option<&LangIdentifier> {
        self.name.as_ref()
    }
}

#[derive(Debug, PartialEq)]
struct DocUnit {
    // state is used to help internally track/identify certain statements
    src: String,
    state: DocState,
    unit: DocItem,
    items: Vec<DocItem>,
}

impl DocUnit {
    pub fn new() -> Self {
        Self {
            src: String::new(),
            state: DocState::Norm,
            unit: DocItem::new(),
            items: Vec::new(),
        }
    }

    pub fn get_source(&self) -> &String {
        &self.src
    }

    pub fn get_name(&self) -> Option<&LangIdentifier> {
        self.unit.name.as_ref()
    }

    pub fn as_parent_name(&self) -> Option<&LangIdentifier> {
        self.unit.parent.as_ref()
    }

    /// Checks if this doc unit matches the requested type of documentation `dtype`.`
    pub fn is_type(&self, dtype: DocType) -> bool {
        self.unit.form == dtype
    }

    /// Attempts to extract the first sentence from the unit's description.
    pub fn get_first_sentence(&self) -> Option<String> {
        self.unit.get_first_sentence()
    }

    /// Writes the unit to it's markdown formatted string.
    pub fn to_markdown(
        &self,
        src_file: Option<String>,
        unit: &LangUnit,
        subs: Vec<&&DocUnit>,
    ) -> String {
        let mut contents = String::new();

        // write the header
        contents.push_str(&format!(
            "# {:?} {}\n",
            self.unit.form,
            self.get_name().unwrap()
        ));

        // link to the source file
        if let Some(src) = src_file {
            contents.push_str(&format!(
                "[Source ({})]({})\n\n",
                unit.get_lang().to_proper_name(),
                src
            ));
        }

        // write the full documentation comment
        contents.push_str(&format!(
            "{}\n\n",
            self.unit.get_docs().unwrap_or(&String::new())
        ));

        // write out any generics
        contents.push_str(&self.add_generic_table(&unit));

        // write out any ports
        contents.push_str(&self.add_port_table(&unit));

        // write out any architectures
        contents.push_str(&self.add_arch_section(&unit, &subs));

        match &self.unit.form {
            DocType::Package => {
                contents.push_str(&self.add_types_section());
                contents.push_str(&self.add_fn_section());
            }
            _ => (),
        }

        contents
    }

    /// Adds the types/subtypes section for VHDL packages.
    fn add_types_section(&self) -> String {
        let mut contents = String::new();
        let type_items: Vec<&DocItem> = self
            .items
            .iter()
            .filter(|f| f.form == DocType::Subtype || f.form == DocType::Type)
            .collect();
        let section = "Types";
        if type_items.len() > 0 {
            contents.push_str(&format!("## {}\n\n", section));
        }
        for item in type_items {
            contents.push_str(&format!("``` vhdl\n{}\n```\n\n{}\n\n", item.stmt, {
                if let Some(docs) = item.get_docs() {
                    format!("> {}", docs.replace("\n", "\n> "))
                } else {
                    String::new()
                }
            }));
        }
        contents
    }

    /// Adds the functions section for VHDL packages.
    fn add_fn_section(&self) -> String {
        let mut contents = String::new();
        let type_items: Vec<&DocItem> = self
            .items
            .iter()
            .filter(|f| f.form == DocType::Function)
            .collect();
        let section = "Functions";
        if type_items.len() > 0 {
            contents.push_str(&format!("## {}\n\n", section));
        }
        for item in type_items {
            contents.push_str(&format!("``` vhdl\n{}\n```\n\n{}\n\n", item.stmt, {
                if let Some(docs) = item.get_docs() {
                    format!("> {}", docs.replace("\n", "\n> "))
                } else {
                    String::new()
                }
            }));
        }
        contents
    }

    /// Adds the architecture section for VHDL entities.
    ///
    /// NOTE: This section still needs to have a source link to it as architectures can be
    /// found in different files from the primrary entity.
    fn add_arch_section(&self, unit: &LangUnit, subs: &Vec<&&DocUnit>) -> String {
        let mut contents = String::new();
        let filtered_subs: Vec<&&&DocUnit> = subs
            .iter()
            .filter(|f| f.is_type(DocType::Architecture))
            .collect();
        let section = "Architectures";

        if let Some(vhd) = unit.get_vhdl_symbol() {
            if let Some(ent) = vhd.as_entity() {
                let archs = ent.get_architectures().inner();
                if archs.len() == 0 {
                    return String::new();
                }
                contents.push_str(&format!("## {}\n\n", section));
                for arch in archs {
                    contents.push_str(&format!("### Architecture {}\n", arch.get_name()));
                    // find this architecture within the doc comments
                    let sel_du = filtered_subs.iter().find(|f| {
                        f.get_name().unwrap() == &LangIdentifier::Vhdl(arch.get_name().clone())
                    });
                    if let Some(du_arch) = sel_du {
                        contents.push_str(&format!(
                            "[Source (VHDL)]({})\n\n",
                            DocProject::get_hdl_source_md_name(du_arch.get_source()).0
                        ));
                        contents.push_str(&format!(
                            "{}\n\n",
                            du_arch.unit.get_docs().unwrap_or(&String::new())
                        ));
                    }
                }
                contents.push_str("\n");
            }
        }

        contents
    }

    fn add_generic_table(&self, unit: &LangUnit) -> String {
        let mut contents = String::new();
        let filtered_dis: Vec<&DocItem> = self
            .items
            .iter()
            .filter(|f| &f.form == &DocType::Generic)
            .collect();
        let section = "Generics";

        contents.push_str(&format!("## {}\n", section));
        contents.push_str(&format!(
            "| Name | Type | Default | Description |\n| :--: | :--: | :--: | :-- |\n"
        ));

        if let Some(vhdl) = unit.get_vhdl_symbol() {
            if vhdl.as_entity().is_none() {
                return String::new();
            }
            let generics = vhdl.as_entity().unwrap().get_generics();
            if generics.0.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for generic in &generics.0 .0 {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt
                        .contains_identifier(LangIdentifier::Vhdl(generic.get_name().clone()))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    generic.get_name(),
                    generic.get_type().to_norm_string(),
                    {
                        if generic.get_default().to_norm_string().len() > 0 {
                            format!("`{}`", generic.get_default().to_norm_string())
                        } else {
                            String::new()
                        }
                    },
                    comment.replace("\n", "<br>")
                ));
            }
        } else if let Some(sv) = unit.get_systemverilog_symbol() {
            if sv.as_module().is_none() {
                return String::new();
            }
            let generics = sv.as_module().unwrap().get_params();
            if generics.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for generic in generics {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt.contains_identifier(LangIdentifier::SystemVerilog(
                        generic.get_name().clone(),
                    ))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    generic.get_name(),
                    generic.get_datatype().to_norm_string(),
                    {
                        if generic.get_default().to_norm_string().len() > 0 {
                            format!("`{}`", generic.get_default().to_norm_string())
                        } else {
                            String::new()
                        }
                    },
                    comment.replace("\n", "<br>")
                ));
            }
        } else if let Some(vlg) = unit.get_verilog_symbol() {
            if vlg.as_module().is_none() {
                return String::new();
            }
            let generics = vlg.as_module().unwrap().get_params();
            if generics.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for generic in generics {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt.contains_identifier(LangIdentifier::SystemVerilog(
                        generic.get_name().clone(),
                    ))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | {} | {} |\n",
                    generic.get_name(),
                    generic.get_datatype().to_norm_string(),
                    {
                        if generic.get_default().to_norm_string().len() > 0 {
                            format!("`{}`", generic.get_default().to_norm_string())
                        } else {
                            String::new()
                        }
                    },
                    comment.replace("\n", "<br>")
                ));
            }
        } else {
            return String::new();
        }
        contents.push_str("\n");
        contents
    }

    fn add_port_table(&self, unit: &LangUnit) -> String {
        let mut contents = String::new();
        let filtered_dis: Vec<&DocItem> = self
            .items
            .iter()
            .filter(|f| &f.form == &DocType::Port)
            .collect();
        let section = "Ports";

        contents.push_str(&format!("## {}\n", section));
        contents.push_str(&format!(
            "| Name | Mode | Type | Description |\n| :--: | :--: | :--: | :-- |\n"
        ));

        if let Some(vhdl) = unit.get_vhdl_symbol() {
            if vhdl.as_entity().is_none() {
                return String::new();
            }
            let ports = vhdl.as_entity().unwrap().get_ports();
            if ports.0.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for port in &ports.0 .0 {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt
                        .contains_identifier(LangIdentifier::Vhdl(port.get_name().clone()))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} |\n",
                    port.get_name(),
                    port.get_mode().to_norm_string(),
                    port.get_type().to_norm_string(),
                    comment.replace("\n", "<br>")
                ));
            }
        } else if let Some(sv) = unit.get_systemverilog_symbol() {
            if sv.as_module().is_none() {
                return String::new();
            }
            let ports = sv.as_module().unwrap().get_ports();
            if ports.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for port in ports {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt
                        .contains_identifier(LangIdentifier::SystemVerilog(port.get_name().clone()))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} |\n",
                    port.get_name(),
                    port.get_mode()
                        .as_ref()
                        .unwrap_or(&SvKeyword::Input)
                        .to_string(),
                    port.get_datatype().to_norm_string(),
                    comment.replace("\n", "<br>")
                ));
            }
        } else if let Some(vlg) = unit.get_verilog_symbol() {
            if vlg.as_module().is_none() {
                return String::new();
            }
            let ports = vlg.as_module().unwrap().get_ports();
            if ports.len() == 0 {
                return String::new();
            }
            // iterate through generics and identify what doc item is linked to the generic
            for port in ports {
                let comment = match filtered_dis.iter().find(|p| {
                    p.stmt
                        .contains_identifier(LangIdentifier::SystemVerilog(port.get_name().clone()))
                }) {
                    Some(di) => match &di.docs {
                        Some(com) => Doc::fix_sentence(&com),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                contents.push_str(&format!(
                    "| `{}` | `{}` | `{}` | {} |\n",
                    port.get_name(),
                    port.get_mode()
                        .as_ref()
                        .unwrap_or(&SvKeyword::Input)
                        .to_string(),
                    port.get_datatype().to_norm_string(),
                    comment.replace("\n", "<br>")
                ));
            }
        } else {
            return String::new();
        }
        contents.push_str("\n");
        contents
    }
}

// VHDL-specific functions
impl DocUnit {
    pub fn from_vhdl_stmt(
        src: String,
        name: LangIdentifier,
        parent: Option<LangIdentifier>,
        cmt: Option<String>,
        stmt: VhdlStatement,
        form: DocType,
    ) -> Self {
        Self {
            state: DocState::Norm,
            src: src,
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
                            name: None,
                            parent: None,
                            form: DocType::Generic,
                            docs: cmt,
                            stmt: Statement::Vhdl(stmt),
                        });
                    } else {
                        self.items.push(DocItem {
                            name: None,
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
        src: String,
        name: LangIdentifier,
        parent: Option<LangIdentifier>,
        cmt: Option<String>,
        stmt: SvStatement,
        form: DocType,
    ) -> Self {
        Self {
            src: src,
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
