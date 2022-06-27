// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    install - store an immutable reference to an ip

SYNOPSIS
    orbit install [options] <ip>

DESCRIPTION
    This command will get move an ip's project folder to the user defined cache.
    By default, the specified version is the 'latest' released version.

OPTIONS
    --ver, -v <version>  
          Version to install
     
    --path <path>  
          Filesystem path to the ip
     
    --git <url>  
          Url to git remote repository for the ip

EXAMPLES
    orbit install ks-tech.rary.gates --ver 1.0.0
    orbit install --git https://github.com/c-rus/gates.git -v latest
";