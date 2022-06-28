// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    install - store an immutable reference to an ip

SYNOPSIS
    orbit install [options]

DESCRIPTION
    This command will get move an ip's project folder to the user defined cache.
    By default, the specified version is the 'latest' released version orbit can
    identify.
      
    An ip can be installed from multiple locations. A common method is to
    reference the ip with its pkgid if it is already in your ip catalog. Another
    method is to install by providing the remote git repository url to clone.
    A third method is to provide the local filesystem path to the ip.
      
    The version is the \"snapshot\" of the ip's state during that time of
    development. Versions are recognized by git tags following the semver
    specification (major.minor.patch).

OPTIONS
    --ip <ip>  
          PkgID to access an orbit ip to install
     
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