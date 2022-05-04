// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    new - create a new ip

SYNOPSIS
    orbit new [options] <ip>

DESCRIPTION
    This command will create a new IP package. The default destination path is
    $ORBIT_PATH/<vendor>/<library>/<name>. If the ORBIT_PATH environment
    variable is not set nor is the core.path key in the config.toml, Orbit
    will use the command's relative path to create the corresponding
    directories.

OPTIONS
    --force  
      Removes the destination directory if it already exists
      
    --path <path>  
      Specify the destination path

EXAMPLES
    orbit new space-tech.rary.gates
";