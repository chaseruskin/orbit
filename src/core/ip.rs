use std::collections::{HashSet, HashMap};
use std::path::PathBuf;

use tempfile::tempdir;

use crate::core::catalog::CacheSlot;
use crate::core::manifest;
use crate::util::graphmap::GraphMap;
use crate::util::overdetsys;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::{AnyError, Fault};
use super::catalog::Catalog;
use super::manifest::IpManifest;
use super::pkgid::PkgId;
use super::version::Version;
use super::vhdl::dst;
use super::vhdl::token::{Identifier, VHDLTokenizer};

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

/// Constructs a graph at the IP-level.
/// 
/// Note: this function performs no reduction.
fn graph_ip<'a>(root: &'a IpManifest, catalog: &'a Catalog<'a>) -> Result<GraphMap<IpSpec, DynState<'a>, ()>, Fault> {
    // create empty graph
    let mut g = GraphMap::new();
    // construct iterative approach with lists
    let t = g.add_node(root.into_ip_spec(), DynState::Keep(root, None));
    let mut processing = vec![(t, root)];

    let mut iden_set: HashSet<Identifier> = HashSet::new();
    // add root's identifiers
    root.collect_units()
        .into_iter()
        .for_each(|u| { iden_set.insert(u.as_iden().unwrap().clone()); } );

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
                                let dst = dep.collect_units()
                                    .into_iter()
                                    .find(|f| iden_set.contains(f.as_iden().unwrap()))
                                    .is_some();
                                
                                // update the hashset with the new unique non-taken identifiers
                                if dst == false {
                                    for unit in dep.collect_units() {
                                        iden_set.insert(unit.as_iden().unwrap().clone());
                                    }
                                }
                                g.add_node(dep.into_ip_spec(), match dst { true => DynState::Alter(dep, None), false => DynState::Keep(dep, None) })
                            };
                            g.add_edge_by_index(s, num, ());
                            processing.push((s, dep));
                        },
                        // try to use the lock file to fill in missing pieces
                        None => panic!("ip is not installed"),
                    }
                },
                // try to use the lock file to fill in missing pieces
                None => return Err(AnyError(format!("unknown ip: {}", pkgid)))?,
            }
        }
    }
    // println!("{:?}", iden_set);
    Ok(g)
}


pub fn compute_final_ip_graph<'a>(target: &'a IpManifest, catalog: &'a Catalog<'a>) -> Result<GraphMap<IpSpec, DynState<'a>, ()>, Fault> {
    // collect rough outline of ip graph
    let mut rough_ip_graph = graph_ip(&target, &catalog)?;
    
    // keep track of list of neighbors that must perform dst and their lookuptables to use after processing all direct impacts
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
                    // println!("neighbor {:?}", rough_ip_graph.get_key_by_index(i));
                    // remember units if true that a transform occurred on the direct conflict node
                    let lut = node.as_ref().as_ip().generate_dst_lut();
                    // determine the neighboring node's ip spec
                    let neighbor_key = rough_ip_graph.get_key_by_index(i).unwrap();
                    // println!("{:?}", lut);
                    match transforms.get_mut(&neighbor_key) {
                        // update the hashmap for the key
                        Some(entry) => lut.into_iter().for_each(|pair| { entry.insert(pair.0, pair.1); () }),
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
pub fn build_ip_file_list<'a>(ip_graph: &'a GraphMap<IpSpec, DynState<'a>, ()>) -> Vec<IpFileNode<'a>> {
    let mut files = Vec::new();
    ip_graph.get_map().iter().for_each(|(_, ip)| {
        crate::util::filesystem::gather_current_files(&ip.as_ref().as_ip().get_root())
            .into_iter()
            .filter(|f| crate::core::fileset::is_vhdl(f) )
            .for_each(|f| {
                files.push(IpFileNode { file: f, ip: ip.as_ref().as_ip() });
            })
    });
    files
}

#[derive(Debug, PartialEq)]
pub enum DynState<'a> {
    Keep(&'a IpManifest, Option<IpManifest>),
    Alter(&'a IpManifest, Option<IpManifest>),
}

impl<'a> DynState<'a> {
    /// References the internal `IpManifest` struct.
    /// 
    /// Favors the dynamic IP if it exists over the original IP.
    pub fn as_ip(&'a self) -> &'a IpManifest {
        match self {
            Self::Keep(ip, t) => {
                if let Some(altered) = t {
                    altered
                } else {
                    ip
                }
            },
            Self::Alter(ip, t) => {
                if let Some(altered) = t {
                    altered
                } else {
                    ip
                }
            }
        }
    }

    /// Checks if an ip is a direct result requiring DST.
    fn is_direct_conflict(&self) -> bool {
        match self {
            Self::Keep(_, _) => false,
            Self::Alter(_, _) => true,
        }
    }

    /// Transforms the current IP into a different installed ip with alternated symbols.
    /// 
    /// Returns the new IpManifest to be replaced with. If the manifest was marked as `Keep`, then
    /// it returns the original manifest.
    /// 
    /// Note: this function can only be applied ip that are already installed to the cache.
    fn dynamic_symbol_transform(&mut self, lut: &HashMap<Identifier, String>, cache_path: &PathBuf) -> () {
        match self {
            Self::Keep(ip, opt) | 
            Self::Alter(ip, opt) => {
                // create a temporary directory
                let temp = tempdir().unwrap();
                let temp_path = temp.path().to_path_buf();
                // copy entire project folder to temporary directory
                crate::util::filesystem::copy(&ip.get_root(), &temp_path).unwrap();

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
                let transformed_ip = install_dst(&temp_ip, &cache_path);
                *opt = Some(transformed_ip);
            },
        }
    }
}

/// Creates a ip manifest that undergoes dynamic symbol transformation.
/// 
/// Returns the DST ip for reference.
fn install_dst(source_ip: &IpManifest, root: &std::path::PathBuf) -> IpManifest {
    // compute the new checksum
    let sum = source_ip.compute_checksum();
    // println!("{}", sum);

    // determine the cache slot name
    let cache_path = {
        let cache_slot = CacheSlot::form(source_ip.get_pkgid().get_name(), source_ip.get_version(), &sum);
        root.join(cache_slot.as_ref())
    };

    // check if already exists and return early with manifest if exists
    if cache_path.exists() == true {
        return IpManifest::from_path(&cache_path).unwrap()
    }

    // copy the source ip to the new location
    crate::util::filesystem::copy(&source_ip.get_root(), &cache_path).unwrap();

    let mut cached_ip = IpManifest::from_path(&cache_path).unwrap();
    cached_ip.stash_units();
    cached_ip.set_as_dynamic();
    // write the new ORBIT_METADATA_FILE
    cached_ip.write_metadata().unwrap();

    // write the new ORBIT_CHECKSUM_FILE
    std::fs::write(&cache_path.join(manifest::ORBIT_SUM_FILE), sum.to_string().as_bytes()).unwrap();

    cached_ip
}

#[derive(Debug, PartialEq)]
pub struct IpFileNode<'a> {
    file: String,
    ip: &'a IpManifest
}

impl<'a> IpFileNode<'a> {
    pub fn new(file: String, ip: &'a IpManifest) -> Self {
        Self { file: file, ip: ip }
    }

    pub fn get_file(&self) -> &String {
        &self.file
    }

    pub fn get_ip_manifest(&self) -> &IpManifest {
        &self.ip
    }

    /// References the library identifier from the ip's pkgid.
    pub fn get_library(&self) -> &PkgPart {
        &self.ip.get_pkgid().get_library().as_ref().unwrap()
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
        write!(f, "{} {}", self.0, self.1)
    }
}