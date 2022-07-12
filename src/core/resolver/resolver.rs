//! dependency resolution

use super::mvs::Module;
use crate::core::manifest::IpManifest;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::Fault;
use crate::util::graph::Graph;

fn build_graph(target: &IpManifest) -> Result<Graph<Module<PkgId>, IpManifest>, Fault> {
    let _ = target.get_dependencies();
    todo!()
}


struct IpSpec(Module<PkgId>);