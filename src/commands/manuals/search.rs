// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    search - browse the ip catalog

SYNOPSIS
    orbit search [options]

DESCRIPTION
    This command will search for all ip defined by Orbit in 3 locations.

OPTIONS
    --cache, -c  
      Filter for ip installed to the cache
     
    --develop, -d  
      Filter for ip in-development within the orbit path
     
    --available, -a  
      Filter for ip available via registries

EXAMPLES
    orbit search --develop --cache --available
";