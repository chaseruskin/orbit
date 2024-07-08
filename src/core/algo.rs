use std::collections::HashMap;
use std::path::PathBuf;

use crate::util::anyerror::{AnyError, CodeFault, Fault};
use crate::util::graphmap::GraphMap;
use std::hash::Hash;
use tempfile::tempdir;

use crate::core::lang::vhdl::primaryunit::VhdlIdentifierError;
use crate::core::lang::vhdl::token::VhdlTokenizer;

use crate::core::catalog::CacheSlot;
use crate::core::catalog::Catalog;
use crate::core::ip::Ip;
use crate::core::ip::IpSpec;
use crate::core::lockfile::{LockEntry, LockFile};
use crate::core::manifest;
use crate::core::version::AnyVersion;

use super::fileset;
use super::lang::verilog::token::tokenizer::VerilogTokenizer;
use super::lang::{verilog, vhdl, Lang, LangIdentifier};
use crate::core::lang::Language;

/// Constructs an ip-graph from a lockfile.
pub fn graph_ip_from_lock(lock: &LockFile) -> Result<GraphMap<IpSpec, &LockEntry, ()>, Fault> {
    let mut graph = GraphMap::new();
    // add all vertices
    lock.inner().iter().for_each(|f| {
        graph.add_node(f.to_ip_spec(), f);
    });
    // add all edges
    lock.inner().iter().for_each(|upper| {
        // get list of dependencies
        for dep in upper.get_deps() {
            // determine the most compatible entry for this dependency
            let lower = lock
                .get_highest(&dep.get_name(), &AnyVersion::from(dep.get_version()))
                .unwrap();
            graph.add_edge_by_key(&lower.to_ip_spec(), &upper.to_ip_spec(), ());
        }
    });
    Ok(graph)
}

/// Constructs a graph at the IP-level.
///
/// Note: this function performs no reduction.
fn graph_ip<'a>(
    root: &'a Ip,
    catalog: &'a Catalog<'a>,
    mode: &Language,
) -> Result<GraphMap<IpSpec, IpNode<'a>, ()>, CodeFault> {
    // create empty graph
    let mut g = GraphMap::new();
    // construct iterative approach with lists
    let t = g.add_node(
        root.get_man().get_ip().into_ip_spec(),
        IpNode::new_keep(root, LangIdentifier::new_working()),
    );
    let mut processing = vec![(t, root)];

    // add root's identifiers and parse files according to the correct language settings
    let mut unit_map =
        Ip::collect_units(true, root.get_root(), mode, false, root.into_public_list())?;

    let mut is_root: bool = true;

    while let Some((num, ip)) = processing.pop() {
        // load dependencies from manifest
        let reqs = ip.get_man().get_deps_list(is_root);
        // read dependencies
        for (pkgid, version) in reqs {
            match catalog.inner().get(pkgid) {
                Some(status) => {
                    // find this IP to read its dependencies
                    match status.get_install(&AnyVersion::from(version)) {
                        Some(dep) => {
                            // check if node is already in graph ????
                            let s = if let Some(existing_node) =
                                g.get_node_by_key(&dep.get_man().get_ip().into_ip_spec())
                            {
                                existing_node.index()
                            } else {
                                // check if identifiers are already taken in graph
                                let units = Ip::collect_units(
                                    false,
                                    dep.get_root(),
                                    mode,
                                    true,
                                    dep.into_public_list(),
                                )?;
                                let dst = if let Some(dupe) =
                                    units.iter().find(|(key, _)| unit_map.contains_key(key))
                                {
                                    let dupe = unit_map.get(dupe.0).unwrap();
                                    if is_root == true {
                                        return Err(CodeFault(
                                            None,
                                            Box::new(VhdlIdentifierError::DuplicateAcrossDirect(
                                                dupe.get_name().to_string(),
                                                dep.get_man().get_ip().into_ip_spec(),
                                                PathBuf::from(dupe.get_source_file()),
                                                dupe.get_vhdl_symbol()
                                                    .unwrap()
                                                    .get_position()
                                                    .clone(),
                                            )),
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
                                let lib = dep.get_man().get_hdl_library();
                                g.add_node(
                                    dep.get_man().get_ip().into_ip_spec(),
                                    match dst {
                                        true => IpNode::new_alter(dep, lib),
                                        false => IpNode::new_keep(dep, lib),
                                    },
                                )
                            };
                            g.add_edge_by_index(s, num, ());
                            processing.push((s, dep));
                        }
                        // todo: try to use the lock file to fill in missing pieces
                        None => {
                            return Err(CodeFault(
                                None,
                                Box::new(AnyError(format!(
                                    "ip {} is not installed",
                                    IpSpec::from((pkgid.clone(), version.clone()))
                                ))),
                            ))?
                        }
                    }
                }
                // todo: try to use the lock file to fill in missing pieces
                // @TODO: check the queue for this IP and attempt to install
                None => {
                    return Err(CodeFault(
                        None,
                        Box::new(AnyError(format!(
                            "unknown ip {}",
                            IpSpec::from((pkgid.clone(), version.clone()))
                        ))),
                    ))?
                }
            }
        }
        is_root = false;
    }
    // println!("{:?}", iden_set);
    Ok(g)
}

pub fn compute_final_ip_graph<'a>(
    target: &'a Ip,
    catalog: &'a Catalog<'a>,
    mode: &Language,
) -> Result<GraphMap<IpSpec, IpNode<'a>, ()>, CodeFault> {
    // collect rough outline of ip graph (after this function, the correct files according to language are kept)
    let mut rough_ip_graph = graph_ip(&target, &catalog, mode)?;

    // keep track of list of neighbors that must perform dst and their lookup-tables to use after processing all direct impacts
    let mut transforms = HashMap::<IpSpec, HashMap<LangIdentifier, String>>::new();

    // iterate through the graph to find all DST nodes to create their replacements
    {
        let mut graph_iter = rough_ip_graph.get_map().iter();
        while let Some((key, node)) = graph_iter.next() {
            if node.as_ref().is_direct_conflict() == true {
                // remember units if true that a transform occurred
                let lut = node.as_ref().as_ip().generate_dst_lut(mode);
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
                let index = rough_ip_graph.get_node_by_key(&key).unwrap().index();
                let mut dependents = rough_ip_graph.get_graph().successors(index);

                while let Some(i) = dependents.next() {
                    // remember units if true that a transform occurred on the direct conflict node
                    let lut = node.as_ref().as_ip().generate_dst_lut(mode);
                    // determine the neighboring node's ip spec
                    let neighbor_key = rough_ip_graph.get_key_by_index(i).unwrap();

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
    let mut transforms_iter = transforms.into_iter();
    while let Some((key, lut)) = transforms_iter.next() {
        rough_ip_graph
            .get_map_mut()
            .get_mut(&key)
            .unwrap()
            .as_ref_mut()
            .dynamic_symbol_transform(&lut, catalog.get_cache_path());
    }

    Ok(rough_ip_graph)
}

/// Take the ip graph and create the entire space of VHDL files that could be used for the current design.
pub fn build_ip_file_list<'a>(
    ip_graph: &'a GraphMap<IpSpec, IpNode<'a>, ()>,
    working_ip: &Ip,
    mode: &Language,
) -> Vec<IpFileNode<'a>> {
    let mut files = Vec::new();
    ip_graph.get_map().iter().for_each(|(_, ip)| {
        let inner_ip = ip.as_ref().as_ip();
        let pub_list = inner_ip.into_public_list();
        crate::util::filesystem::gather_current_files(&inner_ip.get_root(), false)
            .into_iter()
            .filter(|f| working_ip == inner_ip || pub_list.is_included(f.as_ref()))
            .filter(|f| {
                (fileset::is_vhdl(f) && mode.supports_vhdl())
                    || (fileset::is_verilog(f) && mode.supports_verilog())
                    || (fileset::is_systemverilog(f) && mode.supports_systemverilog())
            })
            .for_each(|f| {
                files.push(IpFileNode::new(
                    f,
                    inner_ip,
                    ip.as_ref().get_library().clone(),
                ));
            })
    });
    files
}

#[derive(Debug, PartialEq)]
pub struct IpNode<'a> {
    dyn_state: DynState,
    original: &'a Ip,
    transform: Option<Ip>,
    library: LangIdentifier,
}

#[derive(Debug, PartialEq)]
pub enum DynState {
    Keep,
    Alter,
}

impl<'a> IpNode<'a> {
    fn new_keep(og: &'a Ip, lib: LangIdentifier) -> Self {
        Self {
            dyn_state: DynState::Keep,
            original: og,
            transform: None,
            library: lib,
        }
    }

    fn new_alter(og: &'a Ip, lib: LangIdentifier) -> Self {
        Self {
            dyn_state: DynState::Alter,
            original: og,
            transform: None,
            library: lib,
        }
    }

    /// References the internal `IpManifest` struct.
    ///
    /// Favors the dynamic IP if it exists over the original IP.
    pub fn as_ip(&'a self) -> &'a Ip {
        if let Some(altered) = &self.transform {
            altered
        } else {
            &self.original
        }
    }

    /// References the underlying original `IpManifest` struct regardless if it has
    /// a transform.
    pub fn as_original_ip(&'a self) -> &'a Ip {
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

    /// Transforms the current IP into a different installed ip with alternated symbols.
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
        let temp_ip = Ip::load(temp_path, false).unwrap();

        // edit all vhdl files
        let files = crate::util::filesystem::gather_current_files(temp_ip.get_root(), false);
        for file in &files {
            // perform dst on the data
            if fileset::is_vhdl(&file) == true {
                // parse into tokens
                let vhdl_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&vhdl_path).unwrap();
                let tokens = VhdlTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = vhdl::dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&vhdl_path, transform).unwrap();
            } else if fileset::is_verilog(&file) == true {
                // parse into tokens
                let verilog_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&verilog_path).unwrap();
                let tokens = VerilogTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = verilog::dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&verilog_path, transform).unwrap();
            } else if fileset::is_systemverilog(&file) == true {
                todo!("dst for systemverilog")
            }
        }
        // update the slot with a transformed IP manifest
        self.transform = Some(install_dst(&temp_ip, &cache_path, &lut));
    }
}

/// Creates a ip manifest that undergoes dynamic symbol transformation.
///
/// Returns the DST ip for reference.
fn install_dst(source_ip: &Ip, root: &PathBuf, mapping: &HashMap<LangIdentifier, String>) -> Ip {
    // compute the new checksum on the new ip and its transformed hdl files
    let sum = Ip::compute_checksum(source_ip.get_root());

    // determine the cache slot name
    let cache_path = {
        let cache_slot = CacheSlot::new(
            source_ip.get_man().get_ip().get_name(),
            source_ip.get_man().get_ip().get_version(),
            &sum,
        );
        root.join(cache_slot.to_string())
    };

    // check if already exists and return early with manifest if exists
    if cache_path.exists() == true {
        return Ip::load(cache_path, false).unwrap();
    }

    // copy the source ip to the new location
    crate::util::filesystem::copy(
        &source_ip.get_root(),
        &cache_path,
        true,
        Some(source_ip.get_files_to_keep()),
    )
    .unwrap();
    let cached_ip = Ip::load(cache_path, false).unwrap();

    // @todo: cache results of primary design unit list
    // cached_ip.stash_units();
    // // indicate this installation is dynamic in the metadata
    cached_ip.set_as_dynamic(mapping);
    // // save and write the new metadata
    // cached_ip.write_metadata().unwrap();

    // write the new checksum file
    std::fs::write(
        &cached_ip.get_root().join(manifest::ORBIT_SUM_FILE),
        sum.to_string().as_bytes(),
    )
    .unwrap();

    cached_ip
}

#[derive(Debug, PartialEq)]
pub struct IpFileNode<'a> {
    file: String,
    library: LangIdentifier,
    ip: &'a Ip,
    lang: Lang,
}

impl<'a> Eq for IpFileNode<'a> {}

impl<'a> Hash for IpFileNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.file.hash(state)
    }
}

impl<'a> IpFileNode<'a> {
    pub fn new(file: String, ip: &'a Ip, lib: LangIdentifier) -> Self {
        let lang = if fileset::is_vhdl(&file) == true {
            Lang::Vhdl
        } else if fileset::is_verilog(&file) == true {
            Lang::Verilog
        } else if fileset::is_systemverilog(&file) == true {
            Lang::SystemVerilog
        } else {
            panic!("unsupported language in ip file node")
        };
        Self {
            file: file,
            ip: ip,
            library: lib,
            lang: lang,
        }
    }

    pub fn get_file(&self) -> &String {
        &self.file
    }

    pub fn get_ip(&self) -> &Ip {
        &self.ip
    }

    pub fn get_language(&self) -> &Lang {
        &self.lang
    }

    /// References the library identifier.
    pub fn get_library(&self) -> LangIdentifier {
        self.ip.get_hdl_library()
    }
}
