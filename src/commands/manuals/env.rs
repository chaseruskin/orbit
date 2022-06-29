// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    env - print orbit environment information

SYNOPSIS
    orbit env [options]

DESCRIPTION
    This command will print environment information set by Orbit during runtime.
    By default, it will display all known information in the current 
    environment. 
      
    Optionally passing in keys will print the value's back in the
    order they were accepted on the command line. If a variable does not exist,
    it will print an empty line.

OPTIONS
    <key>...  
          Environment variable keys to request to print

EXAMPLES
    orbit env
    orbit env ORBIT_HOME
    orbit env ORBIT_DEV_PATH ORBIT_HOME
";