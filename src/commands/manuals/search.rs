// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    search - browse the ip catalog

SYNOPSIS
    orbit search [options] [<pkgid>]

DESCRIPTION
    This command will search for all ip defined by Orbit in 3 locations. Use 
    the flags to control what areas to search under (--install, --develop, 
    --available). An optional pkgid can also be provided to narrow results
    even further. Pkgid fields can be omitted by entering an empty value.

OPTIONS
    <pkgid>  
          Identifiers to filter under V.L.N
     
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