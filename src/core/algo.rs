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

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::util::anyerror::{AnyError, CodeFault, Fault};
use crate::util::graphmap::GraphMap;
use crate::warn;
use std::hash::Hash;
use tempfile::tempdir;

use crate::core::lang::vhdl::primaryunit::HdlNamingError;
use crate::core::lang::vhdl::token::VhdlTokenizer;

use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::lockfile::{LockEntry, LockFile};
use crate::core::project::Project;
use crate::core::project::ProjectIdSpec;
use crate::core::version::AnyVersion;

use super::catalog::PkgName;
use super::fileset;
use super::lang::sv::token::tokenizer::SystemVerilogTokenizer;
use super::lang::verilog::token::tokenizer::VerilogTokenizer;
use super::lang::{sv, verilog, vhdl, Lang, LangIdentifier};
use super::project::PartialProjectIdSpec;

/// Constructs a project-level graph from a lockfile.
pub fn graph_project_from_lock(
    lock: &LockFile,
) -> Result<GraphMap<ProjectIdSpec, &LockEntry, ()>, Fault> {
    let mut graph = GraphMap::new();
    // add all vertices
    lock.inner().iter().for_each(|f| {
        graph.add_node(f.to_project_id_spec(), f);
    });
    // add all edges
    lock.inner().iter().for_each(|upper| {
        // get list of dependencies
        for dep in upper.get_deps() {
            // determine the most compatible entry for this dependency
            let lower = lock
                .get_highest(&dep.get_name(), dep.get_version())
                .unwrap();
            graph.add_edge_by_key(&lower.to_project_id_spec(), &upper.to_project_id_spec(), ());
        }
    });
    Ok(graph)
}

/// Constructs a graph at the project-level.
///
/// Note: this function performs no reduction.
fn graph_project<'a>(
    root: &'a Project,
    catalog: Option<&'a Catalog<'a>>,
    private_by_default: bool,
) -> Result<GraphMap<ProjectIdSpec, ProjectNode<'a>, ()>, CodeFault> {
    // create empty graph
    let mut g = GraphMap::new();
    // construct iterative approach with lists
    let t = g.add_node(
        root.get_man().get_project().into_project_id_spec(),
        ProjectNode::new_keep(root, LangIdentifier::new_working()),
    );
    // Only operate on the local ip if catalog is omitted
    if catalog.is_none() {
        return Ok(g);
    }
    let catalog = catalog.unwrap();

    let mut processing = vec![(t, root)];

    // check if we can use the lockfile (is synced with user's manifest)
    let able_to_use_lockfile = root.can_use_lock();

    // add root's identifiers and parse files according to the correct language settings
    let mut unit_map = root.collect_units(true, false, private_by_default)?;

    let mut is_root: bool = true;

    while let Some((num, prj)) = processing.pop() {
        // load dependencies from manifest
        let reqs = prj.get_man().get_deps_list(is_root, true);
        // read dependencies
        for (pkgid, dependency) in reqs {
            // check if we are looking in cache or going local
            match dependency.is_relative() {
                true => {
                    // check if it is a local ip
                    match dependency.as_project() {
                        Some(relative_prj) => {
                            // check if node is already in graph ????
                            let s = if let Some(existing_node) = g.get_node_by_key(
                                &relative_prj.get_man().get_project().into_project_id_spec(),
                            ) {
                                existing_node.index()
                            } else {
                                // check if identifiers are already taken in graph
                                let units =
                                    relative_prj.collect_units(false, true, private_by_default)?;
                                if let Some(dupe) =
                                    units.iter().find(|(key, _)| unit_map.contains_key(key))
                                {
                                    let dupe = unit_map.get(dupe.0).unwrap();
                                    if is_root == true {
                                        return Err(CodeFault(
                                            None,
                                            Box::new(HdlNamingError::DuplicateAcrossDirect(
                                                dupe.get_name().to_string(),
                                                relative_prj
                                                    .get_man()
                                                    .get_project()
                                                    .into_project_id_spec(),
                                                PathBuf::from(
                                                    dupe.get_source_files().first().unwrap(),
                                                ),
                                                dupe.get_position().clone(),
                                            )),
                                        ))?;
                                    } else {
                                        return Err(CodeFault(
                                            None,
                                            Box::new(HdlNamingError::DuplicateAcrossDirect(
                                                dupe.get_name().to_string(),
                                                relative_prj
                                                    .get_man()
                                                    .get_project()
                                                    .into_project_id_spec(),
                                                PathBuf::from(
                                                    dupe.get_source_files().first().unwrap(),
                                                ),
                                                dupe.get_position().clone(),
                                            )),
                                        ))?;
                                    }
                                }
                                // update the hashset with the new unique non-taken identifiers
                                for (key, unit) in units {
                                    unit_map.insert(key, unit);
                                }
                                let lib = relative_prj.get_hdl_library();
                                g.add_node(
                                    relative_prj.get_man().get_project().into_project_id_spec(),
                                    ProjectNode::new_keep(relative_prj, lib),
                                )
                            };
                            g.add_edge_by_index(s, num, ());
                            processing.push((s, &relative_prj));
                        }
                        None => {
                            return Err(CodeFault(
                                None,
                                Box::new(AnyError(format!(
                                    "unknown project {}",
                                    PartialProjectIdSpec::new(
                                        pkgid.clone(),
                                        None,
                                        dependency.get_version().clone()
                                    )
                                ))),
                            ))?
                        }
                    }
                }
                false => {
                    // try to pull uuid from lockfile if it is okay to do so and the manifest is missing the uuid
                    let uuid = match able_to_use_lockfile {
                        true => match root.get_lock().get(pkgid, dependency.get_version()) {
                            Some(entry) => Some(entry.get_uuid()),
                            None => dependency.as_uuid(),
                        },
                        false => dependency.as_uuid(),
                    };
                    // resolve the uuid for this package... try to use existing lockfile from above code segment
                    match catalog.translate_name(&PkgName::new(pkgid, uuid))? {
                        Some(status) => {
                            // find this project to read its dependencies
                            match status.get_install(&AnyVersion::Specific(
                                dependency.get_version().clone(),
                            )) {
                                Some(cached_prj) => {
                                    // check if node is already in graph ????
                                    let s = if let Some(existing_node) = g.get_node_by_key(
                                        &cached_prj.get_man().get_project().into_project_id_spec(),
                                    ) {
                                        existing_node.index()
                                    } else {
                                        // check if identifiers are already taken in graph
                                        let units = cached_prj.collect_units(
                                            false,
                                            true,
                                            private_by_default,
                                        )?;
                                        let dst = if let Some(dupe) =
                                            units.iter().find(|(key, _)| unit_map.contains_key(key))
                                        {
                                            let dupe = unit_map.get(dupe.0).unwrap();
                                            if is_root == true {
                                                return Err(CodeFault(
                                                    None,
                                                    Box::new(
                                                        HdlNamingError::DuplicateAcrossDirect(
                                                            dupe.get_name().to_string(),
                                                            cached_prj
                                                                .get_man()
                                                                .get_project()
                                                                .into_project_id_spec(),
                                                            PathBuf::from(
                                                                dupe.get_source_files()
                                                                    .first()
                                                                    .unwrap(),
                                                            ),
                                                            dupe.get_position().clone(),
                                                        ),
                                                    ),
                                                ))?;
                                            }
                                            true
                                        } else {
                                            false
                                        };
                                        // update the hashset with the new unique non-taken identifiers
                                        if dst == false {
                                            for (key, unit) in units {
                                                unit_map.insert(key, unit);
                                            }
                                        }
                                        let lib = cached_prj.get_hdl_library();
                                        g.add_node(
                                            cached_prj
                                                .get_man()
                                                .get_project()
                                                .into_project_id_spec(),
                                            match dst {
                                                true => ProjectNode::new_alter(cached_prj, lib),
                                                false => ProjectNode::new_keep(cached_prj, lib),
                                            },
                                        )
                                    };
                                    g.add_edge_by_index(s, num, ());
                                    processing.push((s, cached_prj));
                                }
                                // TODO: try to use the lock file to fill in missing pieces
                                None => {
                                    return Err(CodeFault(
                                        None,
                                        Box::new(AnyError(format!(
                                            "project {} is not installed",
                                            PartialProjectIdSpec::new(
                                                pkgid.clone(),
                                                None,
                                                dependency.get_version().clone()
                                            )
                                        ))),
                                    ))?
                                }
                            }
                        }
                        // todo: try to use the lock file to fill in missing pieces
                        // @TODO: check the queue for this project and attempt to install
                        None => {
                            return Err(CodeFault(
                                None,
                                Box::new(AnyError(format!(
                                    "unknown project {}",
                                    PartialProjectIdSpec::new(
                                        pkgid.clone(),
                                        None,
                                        dependency.get_version().clone()
                                    )
                                ))),
                            ))?
                        }
                    }
                }
            }
        }
        is_root = false;
    }
    // println!("{:?}", iden_set);
    Ok(g)
}

pub fn compute_final_project_graph<'a>(
    target: &'a Project,
    catalog: Option<&'a Catalog<'a>>,
    private_by_default: bool,
) -> Result<GraphMap<ProjectIdSpec, ProjectNode<'a>, ()>, CodeFault> {
    // collect rough outline of ip graph (after this function, the correct files according to language are kept)
    let mut rough_project_graph = graph_project(&target, catalog, private_by_default)?;

    // keep track of list of neighbors that must perform dst and their lookup-tables to use after processing all direct impacts
    let mut transforms = HashMap::<ProjectIdSpec, HashMap<LangIdentifier, String>>::new();

    // iterate through the graph to find all DST nodes to create their replacements
    {
        let mut graph_iter = rough_project_graph.get_map().iter();

        while let Some((key, node)) = graph_iter.next() {
            if node.as_ref().is_direct_conflict() == true {
                // remember units if true that a transform occurred
                let lut = node.as_ref().as_project().generate_dst_lut();
                match transforms.get_mut(key) {
                    // update the hashmap for the key
                    Some(entry) => lut.into_iter().for_each(|pair| {
                        entry.insert(pair.0, pair.1);
                        ()
                    }),
                    // create new entry with the lut
                    None => {
                        transforms.insert(key.clone(), lut);
                        ()
                    }
                }

                // grab neighbors and update their hashmaps
                let index = rough_project_graph.get_node_by_key(&key).unwrap().index();
                let mut dependents = rough_project_graph.get_graph().successors(index);

                while let Some(i) = dependents.next() {
                    // remember units if true that a transform occurred on the direct conflict node
                    let lut = node.as_ref().as_project().generate_dst_lut();
                    // determine the neighboring node's ip spec
                    let neighbor_key = rough_project_graph.get_key_by_index(i).unwrap();

                    match transforms.get_mut(&neighbor_key) {
                        // update the hashmap for the key
                        Some(entry) => lut.into_iter().for_each(|pair| {
                            entry.insert(pair.0, pair.1);
                            ()
                        }),
                        // create new entry with the lut
                        None => {
                            transforms.insert(neighbor_key.clone(), lut);
                            ()
                        }
                    }
                }
            }
        }
    }
    // println!("{:?}", transforms);

    // perform each dynamic symbol transform
    if let Some(catalog) = catalog {
        let mut transforms_iter = transforms.into_iter();
        while let Some((key, lut)) = transforms_iter.next() {
            rough_project_graph
                .get_map_mut()
                .get_mut(&key)
                .unwrap()
                .as_ref_mut()
                .dynamic_symbol_transform(&lut, catalog.get_cache_path());
        }
    }

    Ok(rough_project_graph)
}

/// Take the project graph and create the entire space of HDL files that could be used for the current design.
pub fn build_project_file_list<'a>(
    project_graph: &'a GraphMap<ProjectIdSpec, ProjectNode<'a>, ()>,
    current_project: &Project,
) -> Vec<ProjectFileNode<'a>> {
    let mut files = Vec::new();
    project_graph.get_map().iter().for_each(|(_, prj)| {
        let inner_prj = prj.as_ref().as_project();
        let non_private_list = inner_prj.into_non_private_list();
        inner_prj
            .gather_current_files()
            .into_iter()
            .filter(|f| {
                current_project == inner_prj
                    || inner_prj.get_mapping().is_relative()
                    || non_private_list.is_included(f.as_ref())
            })
            .filter(|f| {
                (fileset::is_vhdl(f)) || (fileset::is_verilog(f)) || (fileset::is_systemverilog(f))
            })
            .for_each(|f| {
                files.push(ProjectFileNode::new(
                    f,
                    inner_prj,
                    prj.as_ref().get_library().clone(),
                ));
            })
    });
    // sort the files such that things are deterministic across runs moving
    // forward when performing topological sorting
    files.sort();
    files
}

/// Create a minimal graph map that consists of just this local ip node.
///
/// Useful for initializing or creating new ip and having to make the lockfile.
pub fn minimal_graph_map<'a>(
    current_prj: &'a Project,
) -> GraphMap<ProjectIdSpec, ProjectNode<'a>, ()> {
    let mut g = GraphMap::new();
    g.add_node(
        current_prj.get_man().get_project().into_project_id_spec(),
        ProjectNode::new_keep(current_prj, LangIdentifier::new_working()),
    );
    g
}

#[derive(Debug, PartialEq)]
pub struct ProjectNode<'a> {
    dyn_state: DynState,
    original: &'a Project,
    transform: Option<Project>,
    library: LangIdentifier,
}

#[derive(Debug, PartialEq)]
pub enum DynState {
    Keep,
    Alter,
}

impl<'a> ProjectNode<'a> {
    fn new_keep(og: &'a Project, lib: LangIdentifier) -> Self {
        Self {
            dyn_state: DynState::Keep,
            original: og,
            transform: None,
            library: lib,
        }
    }

    fn new_alter(og: &'a Project, lib: LangIdentifier) -> Self {
        Self {
            dyn_state: DynState::Alter,
            original: og,
            transform: None,
            library: lib,
        }
    }

    /// References the internal `IpManifest` struct.
    ///
    /// Favors the dynamic project if it exists over the original project.
    pub fn as_project(&'a self) -> &'a Project {
        if let Some(altered) = &self.transform {
            altered
        } else {
            &self.original
        }
    }

    /// References the underlying original `IpManifest` struct regardless if it has
    /// a transform.
    pub fn as_original_project(&'a self) -> &'a Project {
        &self.original
    }

    fn get_library(&self) -> &LangIdentifier {
        &self.library
    }

    /// Checks if an ip is a direct result requiring DST.
    fn is_direct_conflict(&self) -> bool {
        match &self.dyn_state {
            DynState::Alter => true,
            DynState::Keep => false,
        }
    }

    /// Transforms the current project into a different installed ip with alternated symbols.
    ///
    /// Returns the new IpManifest to be replaced with. If the manifest was marked as `Keep`, then
    /// it returns the original manifest.
    ///
    /// Note: this function can only be applied ip that are already installed to the cache.
    fn dynamic_symbol_transform(
        &mut self,
        lut: &HashMap<LangIdentifier, String>,
        cache_path: &PathBuf,
    ) -> () {
        // create a temporary directory
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();
        // copy entire project folder to temporary directory
        crate::util::filesystem::copy(
            &self.original.get_root(),
            &temp_path,
            true,
            Some(self.original.get_files_to_keep()),
        )
        .unwrap();

        // create the ip from the temporary dir
        let temp_prj = Project::load(temp_path, false, false).unwrap();

        // edit all vhdl files
        let files = temp_prj.gather_current_files();
        for file in &files {
            // perform dst on the data (VHDL)
            if fileset::is_vhdl(&file) == true {
                // parse into tokens
                let vhdl_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&vhdl_path).unwrap();
                let tokens = VhdlTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = vhdl::dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&vhdl_path, transform).unwrap();
            // HANDLE VERILOG DST ALGORITHM
            } else if fileset::is_verilog(&file) == true {
                // parse into tokens
                let verilog_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&verilog_path).unwrap();
                let tokens = VerilogTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = verilog::dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&verilog_path, transform).unwrap();
            // handle SV DST ALGORITHM
            } else if fileset::is_systemverilog(&file) == true {
                // parse into tokens
                let systemverilog_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&systemverilog_path).unwrap();
                let tokens = SystemVerilogTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = sv::dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&systemverilog_path, transform).unwrap();
            }
        }
        // update the slot with a transformed project manifest
        self.transform = Some(install_dst(
            &temp_prj,
            &cache_path,
            &lut,
            self.original.get_mapping().is_relative(),
        ));
    }
}

/// Creates a ip manifest that undergoes dynamic symbol transformation.
///
/// Returns the DST ip for reference.
fn install_dst(
    source_prj: &Project,
    root: &PathBuf,
    mapping: &HashMap<LangIdentifier, String>,
    is_rel: bool,
) -> Project {
    // compute the new checksum on the new ip and its transformed hdl files
    let sum = Project::compute_checksum(source_prj.get_root());

    // determine the cache slot name
    let cache_slot = if is_rel == true {
        warn!(
            "using dynamic cache slot entry for relative dependency {}",
            source_prj.get_man().get_project().get_name()
        );
        CacheSlot::new_rel(
            source_prj.get_uuid(),
            source_prj.get_man().get_project().get_version(),
        )
    } else {
        CacheSlot::new(
            source_prj.get_uuid(),
            source_prj.get_man().get_project().get_version(),
            &sum,
        )
    };
    let cache_path = root.join(cache_slot.to_string());

    // check if already exists and return early with manifest if exists
    if cache_path.exists() == true && is_rel == false {
        return Project::load(cache_path, false, false).unwrap();
    }

    if is_rel == false {
        // copy the source ip to the new location
        crate::util::filesystem::copy(
            &source_prj.get_root(),
            &cache_path,
            true,
            Some(source_prj.get_files_to_keep()),
        )
        .unwrap();
    } else {
        // perform a "smart" copy to the location for relative DST
        crate::util::filesystem::smart_copy(
            &source_prj.get_root(),
            &cache_path,
            true,
            Some(source_prj.get_files_to_keep()),
        )
        .unwrap();
    }

    // clean up temporary directory
    std::fs::remove_dir_all(&source_prj.get_root()).unwrap();

    let cached_prj = match Project::load(cache_path.clone(), false, false) {
        Ok(r) => r,
        Err(e) => {
            // clean up corrupt cache entry directory
            std::fs::remove_dir_all(&cache_path).unwrap();
            panic!(
                "an unexpected error occurred during the final copy of a dynamic cache entry: {}",
                e
            );
        }
    };
    // indicate this installation is dynamic in the metadata
    cached_prj.set_as_dynamic(mapping);
    // write the new checksum file
    cached_prj.write_cache_checksum(&sum).unwrap();
    // write the metadata
    cached_prj.write_cache_metadata().unwrap();

    cached_prj
}

#[derive(Debug, Clone)]
pub struct ProjectFileNode<'a> {
    file: String,
    library: LangIdentifier,
    project: &'a Project,
    lang: Lang,
    dep_files: Vec<String>,
}

impl<'a> Ord for ProjectFileNode<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.file).cmp(&(other.file))
    }
}

impl<'a> PartialOrd for ProjectFileNode<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for ProjectFileNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        // do not care about the dependency file list when comparing project file nodes
        self.file == other.file
            && self.library == other.library
            && self.project == other.project
            && self.lang == other.lang
    }
}

impl<'a> Eq for ProjectFileNode<'a> {}

impl<'a> Hash for ProjectFileNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file.hash(state)
    }
}

impl<'a> ProjectFileNode<'a> {
    pub fn new(file: String, project: &'a Project, lib: LangIdentifier) -> Self {
        let lang = if fileset::is_vhdl(&file) == true {
            Lang::Vhdl
        } else if fileset::is_verilog(&file) == true {
            Lang::Verilog
        } else if fileset::is_systemverilog(&file) == true {
            Lang::SystemVerilog
        } else {
            panic!("unsupported language in project file node")
        };
        Self {
            file: file,
            project,
            library: lib,
            lang: lang,
            dep_files: Vec::new(),
        }
    }

    pub fn get_file(&self) -> &String {
        &self.file
    }

    pub fn get_project(&self) -> &Project {
        &self.project
    }

    pub fn get_language(&self) -> &Lang {
        &self.lang
    }

    /// References the library identifier.
    pub fn get_library(&self) -> LangIdentifier {
        self.project.get_hdl_library()
    }

    /// Sets the list of direct dependency filepaths.
    pub fn add_dep_files(&mut self, mut deps: Vec<String>) {
        self.dep_files.append(&mut deps);
    }

    pub fn add_dep_file(&mut self, dep: String) {
        self.dep_files.push(dep);
    }

    /// Get the direct dependency filepaths.
    pub fn get_dep_files(&self) -> &Vec<String> {
        &self.dep_files
    }
}
