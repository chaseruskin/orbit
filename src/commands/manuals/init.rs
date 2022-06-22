// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    init - initialize an existing directory as an ip

SYNOPSIS
    orbit init [options] <ip>

DESCRIPTION
    This command will initialize an existing directory/project into a IP
    recognized by Orbit. 
    if the --git option is combined with --path, then the project will be
    cloned to the specified path. 
    If ORBIT_DEV_PATH is set, then path will be relative to the ORBIT_DEV_PATH.

OPTIONS
    <ip>  
          The fully specified pkgid to name to the ip
     
    --git <repo>  
          A git repository to clone
     
    --path <path>  
          A filesystem destination to initialize the ip

EXAMPLES
    orbit init ks-tech.rary.gates --git https://github.com/ks-tech/gates.git
";