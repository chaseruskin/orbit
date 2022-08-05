// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    search - browse the ip catalog

SYNOPSIS
    orbit search [options] [<pkgid>]

DESCRIPTION
    This command will search for all ip defined by Orbit in the catalog from all
    3 state: development, installation, and available. You can control what 
    states to search for with --develop, --install, and --available flags.
      
    An optional pkgid can also be provided to narrow results even further. Pkgid 
    fields can be omitted by entering an empty value.

OPTIONS
    <pkgid>  
          Identifiers to filter under vendory.library.name
     
    --install, -i  
          Filter for ip installed to the cache
     
    --develop, -d  
          Filter for ip in-development within the orbit path
     
    --available, -a  
          Filter for ip available via registries

EXAMPLES
    orbit search --develop --install --available
    orbit search rary. -i
    orbit search gates -ia
    orbit search ks-tecth.rary.gates -d
";