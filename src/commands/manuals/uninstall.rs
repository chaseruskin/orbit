// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    uninstall - remove an ip from the catalog

SYNOPSIS
    orbit uninstall [options] <ip>

DESCRIPTION
    This command will delete the project directory of an IP. By default, Orbit
    will delete the IP found on the DEV_PATH.

OPTIONS
    --variant, -v <version>  
          Access the settings to the home configuration file
     
    --force    
          Remove the ip regardless of conditions

EXAMPLES
    orbit uninstall kepler.rary.gates -v dev
    orbit uninstall kepler.util.toolbox --variant 2.1
";