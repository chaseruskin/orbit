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
      
    The hdl tree only displays the connections between entity in the graph. If
    an unidentified entity is instantiated it will appear as a leaf and is 
    denoted as a black box by a '?' character.
      
    An entity is considered a black box if Orbit cannot find its hdl source code
    file.
      
    The hdl tree does not show how many times an entity is instantiated within 
    a parent entity, and all architectures for each entity are analyzed.
      
    To view the ip dependency tree, use --ip.

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
