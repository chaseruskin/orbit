// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    env - print orbit environment information

SYNOPSIS
    orbit env [options]

DESCRIPTION
    This command prints environment variables relevant to 'orbit'.
    
    By default, this command prins information as a shell script. If one or more
    variable names are given as arguments as '<key>', then it will print the value
    of each provided variables on its own line.

OPTIONS
    <key>...
        Include this variable's value specifically in the environment information

EXAMPLES
    orbit env
    orbit env ORBIT_HOME
    orbit env ORBIT_CACHE ORBIT_ARCHIVE
"#;
