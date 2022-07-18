use crate::util::graph::Graph;
use crate::util::overdetsys;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::{AnyError, Fault};
use super::catalog::Catalog;
use super::manifest::IpManifest;
use super::pkgid::PkgId;

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
pub fn graph_ip<'a>(root: &'a IpManifest, catalog: &'a Catalog) -> Result<Graph<&'a IpManifest, ()>, Fault> {
    // create empty graph
    let mut g = Graph::new();
    // construct iterative approach with lists
    let t = g.add_node(root);
    let mut processing = vec![(t, root)];

    while let Some((num, ip)) = processing.pop() {
        // read dependencies
        let deps = ip.get_dependencies();
        for (pkgid, version) in deps.inner() {
            match catalog.inner().get(pkgid) {
                Some(status) => {
                    // find this IP to read its dependencies
                    match status.get(version) {
                        Some(dep) => {
                            let s = g.add_node(dep);
                            g.add_edge(s, num, ());
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
    Ok(g)
}