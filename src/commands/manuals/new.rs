// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    new - create a new ip

SYNOPSIS
    orbit new [options] <ip>

DESCRIPTION
    This command will create a new IP package. The default destination path is
    $ORBIT_DEV_PATH/<vendor>/<library>/<name>. If the ORBIT_DEV_PATH 
    environment variable is not set and core.path entry is absent from 
    configuration, Orbit will use the directory where the command was invoked as
    the base path.

OPTIONS
    --force  
          Removes the destination directory if it already exists
      
    --path <path>  
          Specify the destination path
      
    --template <alias>  
          Specify a configured template to copy

EXAMPLES
    orbit new ks-tech.rary.gates
    orbit new ks-tech.rary.common --template base --path common
";