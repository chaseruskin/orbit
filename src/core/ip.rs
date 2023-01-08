use std::collections::HashMap;
use std::path::PathBuf;

use tempfile::tempdir;

use crate::core::catalog::CacheSlot;
use crate::core::manifest;
use crate::util::graphmap::GraphMap;
use crate::util::overdetsys;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::{AnyError, Fault};
use super::catalog::Catalog;
use super::lockfile::{LockEntry, LockFile};
use super::manifest::IpManifest;
use super::pkgid::PkgId;
use super::version::Version;
use super::lang::vhdl::dst;
use super::lang::vhdl::primaryunit::{VhdlIdentifierError, PrimaryUnit};
use super::lang::vhdl::token::{Identifier, VHDLTokenizer};

/// Given a partial/full ip specification `ip_spec`, sift through the manifests
/// for a possible determined unique solution.
/// 
/// Note: Currently clones each id, possibly look for faster implemtenation avoiding clone.
pub fn find_ip(ip_spec: &PkgId, universe: Vec<&PkgId>) -> Result<PkgId, AnyError> {
    // try to find ip name
    let space: Vec<Vec<PkgPart>> = universe.into_iter().map(|f| { f.into_full_vec().unwrap() }).collect();
    let result = match overdetsys::solve(space, ip_spec.iter()) {
        Ok(r) => r,
        Err(e) => match e {
            overdetsys::OverDetSysError::NoSolution => Err(AnyError(format!("no ip as '{}' exists", ip_spec)))?,
            overdetsys::OverDetSysError::Ambiguous(set) => {
                // assemble error message
                let mut set = set.into_iter().map(|f| PkgId::from_vec(f) );
                let mut content = String::new();
                while let Some(s) = set.next() {
                    content.push_str(&format!("    {}\n", s.to_string()));
                }
                Err(AnyError(format!("ambiguous ip '{}' yields multiple solutions:\n{}", ip_spec, content)))?
            }
        }
    };
    Ok(PkgId::from_vec(result))
}

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
        if let Some(deps) = upper.get_deps() {
            for dep in deps {
                // determine the most compatible entry for this dependency
                let lower = lock.get_highest(&dep.0, &dep.1).unwrap();
                graph.add_edge_by_key(&lower.to_ip_spec(), &upper.to_ip_spec(), ());
            }
        }
    });
    Ok(graph)
}

/// Constructs a graph at the IP-level.
/// 
/// Note: this function performs no reduction.
fn graph_ip<'a>(root: &'a IpManifest, catalog: &'a Catalog<'a>) -> Result<GraphMap<IpSpec, IpNode<'a>, ()>, Fault> {
    // create empty graph
    let mut g = GraphMap::new();
    // construct iterative approach with lists
    let t = g.add_node(root.into_ip_spec(), IpNode::new_keep(root, Identifier::new_working()));
    let mut processing = vec![(t, root)];
    
    let mut iden_set: HashMap<Identifier, PrimaryUnit> = HashMap::new();
    // add root's identifiers
    root.collect_units(true)?
        .into_iter()
        .for_each(|(key, unit)| { iden_set.insert(key, unit); } );

    let mut is_root: bool = true;

    while let Some((num, ip)) = processing.pop() {
        // read dependencies
        let deps = ip.get_dependencies();
        for (pkgid, version) in deps.inner() {
            match catalog.inner().get(pkgid) {
                Some(status) => {
                    // find this IP to read its dependencies
                    match status.get(version, true) {
                        Some(dep) => {
                            // check if node is already in graph ????
                            let s = if let Some(existing_node) = g.get_node_by_key(&dep.into_ip_spec()) {
                                existing_node.index()
                            } else {
                                // check if identifiers are already taken in graph
                                let units = dep.collect_units(false)?;
                                let dst = if let Some(dupe) = units
                                        .iter()
                                        .find(|(key, _)| iden_set.contains_key(key)) {
                                    let dupe = iden_set.get(dupe.0).unwrap();
                                    if is_root == true {
                                        return Err(VhdlIdentifierError::DuplicateAcrossDirect(
                                            dupe.get_iden().clone(), 
                                            dep.into_ip_spec(),
                                            PathBuf::from(dupe.get_unit().get_source_code_file().clone()),
                                            dupe.get_unit().get_symbol().unwrap().get_position().clone()
                                        ))?
                                    }
                                    true
                                } else {
                                    false
                                };
                                // update the hashset with the new unique non-taken identifiers
                                if dst == false {
                                    for (key, unit) in units {
                                        iden_set.insert(key, unit);
                                    }
                                }
                                let lib = Identifier::from(dep.get_pkgid().get_library().as_ref().unwrap());
                                g.add_node(dep.into_ip_spec(), match dst { true => IpNode::new_alter(dep, lib), false => IpNode::new_keep(dep, lib) })
                            };
                            g.add_edge_by_index(s, num, ());
                            processing.push((s, dep));
                        },
                        // todo: try to use the lock file to fill in missing pieces
                        None => return Err(AnyError(format!("ip '{} v{}' is not installed", pkgid, version)))?,
                    }
                },
                // todo: try to use the lock file to fill in missing pieces
                None => return Err(AnyError(format!("unknown ip: {}", pkgid)))?,
            }
        }
        is_root = false;
    }
    // println!("{:?}", iden_set);
    Ok(g)
}


pub fn compute_final_ip_graph<'a>(target: &'a IpManifest, catalog: &'a Catalog<'a>) -> Result<GraphMap<IpSpec, IpNode<'a>, ()>, Fault> {
    // collect rough outline of ip graph
    let mut rough_ip_graph = graph_ip(&target, &catalog)?;
    
    // keep track of list of neighbors that must perform dst and their lookup-tables to use after processing all direct impacts
    let mut transforms = HashMap::<IpSpec, HashMap<Identifier, String>>::new();

    // iterate through the graph to find all DST nodes to create their replacements
    {
        let mut graph_iter = rough_ip_graph.get_map().iter();
        while let Some((key, node)) = graph_iter.next() {
            if node.as_ref().is_direct_conflict() == true {
                // remember units if true that a transform occurred
                let lut = node.as_ref().as_ip().generate_dst_lut();
                match transforms.get_mut(key) {
                    // update the hashmap for the key
                    Some(entry) => lut.into_iter().for_each(|pair| { entry.insert(pair.0, pair.1); () }),
                    // create new entry with the lut
                    None => { transforms.insert(key.clone(), lut); () },
                }
                
                // grab neighbors and update their hashmaps
                let index = rough_ip_graph.get_node_by_key(&key).unwrap().index();
                let mut dependents = rough_ip_graph.get_graph().successors(index);

                while let Some(i) = dependents.next() {
                    // remember units if true that a transform occurred on the direct conflict node
                    let lut = node.as_ref().as_ip().generate_dst_lut();
                    // determine the neighboring node's ip spec
                    let neighbor_key = rough_ip_graph.get_key_by_index(i).unwrap();
                    
                    match transforms.get_mut(&neighbor_key) {
                        // update the hashmap for the key
                        Some(entry) => lut.into_iter()
                            .for_each(|pair| { entry.insert(pair.0, pair.1); () }),
                        // create new entry with the lut
                        None => { transforms.insert(neighbor_key.clone(), lut); () },
                    }
                }
            }
        }
    }
    // println!("{:?}", transforms);

    // perform each dynamic symbol transform
    let mut transforms_iter = transforms.into_iter();
    while let Some((key, lut)) = transforms_iter.next() {
        rough_ip_graph.get_map_mut().get_mut(&key).unwrap().as_ref_mut().dynamic_symbol_transform(&lut, catalog.get_cache_path());
    }

    Ok(rough_ip_graph)
}

/// Take the ip graph and create the entire space of VHDL files that could be used for the current design.
pub fn build_ip_file_list<'a>(ip_graph: &'a GraphMap<IpSpec, IpNode<'a>, ()>) -> Vec<IpFileNode<'a>> {
    let mut files = Vec::new();
    ip_graph.get_map().iter().for_each(|(_, ip)| {
        crate::util::filesystem::gather_current_files(&ip.as_ref().as_ip().get_root())
            .into_iter()
            .filter(|f| crate::core::fileset::is_vhdl(f) )
            .for_each(|f| {
                files.push(IpFileNode { file: f, ip: ip.as_ref().as_ip(), library: ip.as_ref().get_library().clone() });
            })
    });
    files
}

#[derive(Debug, PartialEq)]
pub struct IpNode<'a> {
    dyn_state: DynState,
    original: &'a IpManifest,
    transform: Option<IpManifest>,
    library: Identifier,
}

#[derive(Debug, PartialEq)]
pub enum DynState {
    Keep,
    Alter
}

impl<'a> IpNode<'a> {
    fn new_keep(og: &'a IpManifest, lib: Identifier) -> Self {
        Self { dyn_state: DynState::Keep, original: og, transform: None, library: lib }
    }

    fn new_alter(og: &'a IpManifest, lib: Identifier) -> Self {
        Self { dyn_state: DynState::Alter, original: og, transform: None, library: lib }
    }

    /// References the internal `IpManifest` struct.
    /// 
    /// Favors the dynamic IP if it exists over the original IP.
    pub fn as_ip(&'a self) -> &'a IpManifest {
        if let Some(altered) = &self.transform {
            altered
        } else {
            &self.original
        }
    }

    /// References the underlying original `IpManifest` struct regardless if it has
    /// a transform.
    pub fn as_original_ip(&'a self) -> &'a IpManifest {
        &self.original
    }

    fn get_library(&self) -> &Identifier {
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
    fn dynamic_symbol_transform(&mut self, lut: &HashMap<Identifier, String>, cache_path: &PathBuf) -> () {
        // create a temporary directory
        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();
        // copy entire project folder to temporary directory
        crate::util::filesystem::copy(&self.original.get_root(), &temp_path, true).unwrap();

        // create the ip from the temporary dir
        let temp_ip = IpManifest::from_path(&temp_path).unwrap();

        // edit all vhdl files
        let files = crate::util::filesystem::gather_current_files(&temp_path);
        for file in &files {
            // perform dst on the data
            if crate::core::fileset::is_vhdl(&file) == true {
                // parse into tokens
                let vhdl_path = PathBuf::from(file);
                let code = std::fs::read_to_string(&vhdl_path).unwrap();
                let tokens = VHDLTokenizer::from_source_code(&code).into_tokens_all();
                // perform DYNAMIC SYMBOL TRANSFORM
                let transform = dst::dyn_symbol_transform(&tokens, &lut);
                // rewrite the file
                std::fs::write(&vhdl_path, transform).unwrap();
            }
        }
        // update the slot with a transformed IP manifest
        self.transform = Some(install_dst(&temp_ip, &cache_path));
    }
}

/// Creates a ip manifest that undergoes dynamic symbol transformation.
/// 
/// Returns the DST ip for reference.
fn install_dst(source_ip: &IpManifest, root: &std::path::PathBuf) -> IpManifest {
    // compute the new checksum on the new ip and its transformed hdl files
    let sum = source_ip.compute_checksum();

    // determine the cache slot name
    let cache_path = {
        let cache_slot = CacheSlot::new(source_ip.get_pkgid().get_name(), source_ip.get_version(), &sum);
        root.join(cache_slot.as_ref())
    };

    // check if already exists and return early with manifest if exists
    if cache_path.exists() == true {
        return IpManifest::from_path(&cache_path).unwrap()
    }

    // copy the source ip to the new location
    crate::util::filesystem::copy(&source_ip.get_root(), &cache_path, true).unwrap();
    let mut cached_ip = IpManifest::from_path(&cache_path).unwrap();

    // cache results of primary design unit list
    cached_ip.stash_units();
    // indicate this installation is dynamic in the metadata
    cached_ip.set_as_dynamic();
    // save and write the new metadata
    cached_ip.write_metadata().unwrap();

    // write the new checksum file
    std::fs::write(&cache_path.join(manifest::ORBIT_SUM_FILE), sum.to_string().as_bytes()).unwrap();

    cached_ip
}

#[derive(Debug, PartialEq)]
pub struct IpFileNode<'a> {
    file: String,
    library: Identifier,
    ip: &'a IpManifest
}

impl<'a> IpFileNode<'a> {
    pub fn new(file: String, ip: &'a IpManifest, lib: Identifier) -> Self {
        Self { file: file, ip: ip, library: lib }
    }

    pub fn get_file(&self) -> &String {
        &self.file
    }

    pub fn get_ip_manifest(&self) -> &IpManifest {
        &self.ip
    }

    /// References the library identifier.
    pub fn get_library(&self) -> &Identifier {
        &self.library
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct IpSpec(PkgId, Version);

impl IpSpec {
    pub fn new(pkgid: PkgId, version: Version) -> Self {
        Self(pkgid, version)
    }
}

impl std::fmt::Display for IpSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.0, self.1)
    }
}