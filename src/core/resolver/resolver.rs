//! dependency resolution

use super::mvs::Module;
use crate::core::pkgid::PkgId;
use crate::util::anyerror::Fault;
use crate::util::graph::Graph;
use crate::core::ip::Ip;

fn build_graph(target: &Ip) -> Result<Graph<Module<PkgId>, Ip>, Fault> {
    let _ = target.get_manifest().get_dependencies();

    todo!();
}


struct IpSpec(Module<PkgId>);