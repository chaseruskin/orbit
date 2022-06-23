// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    query - probe information about an orbit IP

SYNOPSIS
    orbit query [options] <ip>

DESCRIPTION
    This command will print information to the console for the user to learn
    more information about any given IP. By default, it will return the
    Orbit.toml manifest file contents.
      
    Since IP can exist at 3 different levels, the default IP manifest to return
    data about is the latest installed version. If there is none, it will try
    the latest available version, and then the development version, if those 
    exist.

OPTIONS
    <ip>  
          The fully specified pkgid for the ip
     
    --tags  
          Return a list of versions and where they are located
     
    --install, -i <version>  
          Extract data from a manifest stored in the cache at <version>
     
    --available, -a <version>  
          Extract data from a manifest stored in a vendor at <version>
     
    --develop, -d  
          Extract data from a manifest stored on DEV_PATH
     
    --units  
          List the available primary design units within the IP

EXAMPLES
    orbit query ks-tech.rary.gates --tags
    orbit query util.toolbox -i 1.2.3 --units
";