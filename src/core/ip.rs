use std::collections::HashSet;

use crate::util::graphmap::GraphMap;
use crate::util::overdetsys;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::{AnyError, Fault};
use super::catalog::Catalog;
use super::manifest::IpManifest;
use super::pkgid::PkgId;
use super::version::Version;
use super::vhdl::token::Identifier;

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
pub fn graph_ip<'a>(root: &'a IpManifest, catalog: &'a Catalog) -> Result<GraphMap<IpSpec, DynState<'a>, ()>, Fault> {
    // create empty graph
    let mut g = GraphMap::new();
    // construct iterative approach with lists
    let t = g.add_node(root.into_ip_spec(), DynState::Keep(root));
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
                                g.add_node(dep.into_ip_spec(), match dst { true => DynState::Alter(dep), false => DynState::Keep(dep) })
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

    println!("{:?}", iden_set);
    Ok(g)
}


#[derive(Debug, PartialEq)]
pub enum DynState<'a> {
    Keep(&'a IpManifest),
    Alter(&'a IpManifest),
}

impl<'a> DynState<'a> {
    pub fn as_ip(&self) -> &'a IpManifest {
        match self {
            Self::Keep(ip) => ip,
            Self::Alter(ip) => ip,
        }
    }
}



#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct IpSpec(PkgId, Version);

impl IpSpec {
    pub fn new(pkgid: PkgId, version: Version) -> Self {
        Self(pkgid, version)
    }
}