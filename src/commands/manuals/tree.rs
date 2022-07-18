// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    tree - view the dependency graph

SYNOPSIS
    orbit tree [options]

DESCRIPTION
    This command will show the hierarchical tree-structure starting from a node.
    By default, Orbit will attempt to automatically detect the root if it is
    unambiguous.

OPTIONS
    --root <entity>  
          Top-level entity identifier to mark as the root node
     
    --compress  
          Replace duplicate branches with a label marking
     
    --all  
          Include all possible roots in hierarchical tree
     
    --format <fmt>  
          Select how to display entity names: 'long' or 'short'
     
    --ascii  
          Restricts characters to original 128 ascii set
     
    --ip  
          View the ip-level dependency graph

EXAMPLES
    orbit tree --root nor_gate
    orbit tree --ip
";