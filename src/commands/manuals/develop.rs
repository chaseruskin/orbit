// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    develop - bring an ip to the development state

SYNOPSIS
    orbit develop [options]

DESCRIPTION
    This command will get move an ip's project folder to the DEV_PATH. By default,
    the specified version is the 'latest' released version orbit can
    identify.
      
    An ip can be brought to development from multiple sources. A common method is 
    to reference the ip with its pkgid if it is found in the ip catalog with
    --ip. Another method is to provide the remote git repository url to clone
    with --git. A third method is to provide the local filesystem
    path to the ip with --path.
      
    The version is the \"snapshot\" of the ip's state during that time of
    development. Versions are recognized by Orbit as git tags following the 
    semver specification (major.minor.patch).
      
    Different levels of development can be specified. By default, only the ip's
    project will be brought to the DEV_PATH and all dependencies will be
    installed to the cache according to its lockfile. You can also bring direct 
    dependencies to the DEV_PATH with --mode direct. If all dependencies are
    needing development, use --mode all to bring all dependencies to the DEV_PATH.
    This behavior will also modify every manifest to point to \"dev\"
      
    Bringing in dependencies to the DEV_PATH will error if the dependency is 
    already found on the DEV_PATH, or if dynamic symbol resolution is required
    by the current ip. Using --force will overwrite the existing projects in 
    the DEV_PATH and will bring dependencies to the DEV_PATH regardless if DST 
    was applied. 
      
    Upon bringing dependencies to the DEV_PATH, they will be checked out to their
    commit for their corresponding version. 

OPTIONS
    --ip <ip>  
          Pkgid to access an orbit ip to install
     
    --variant, -v <version>  
          Version to install
     
    --path <path>  
          Filesystem path to the ip
     
    --git <url>  
          Url to git remote repository for the ip
     
    --disable-ssh  
          Convert SSH to HTTPS urls when fetching external dependencies

EXAMPLES
    orbit install --ip ks-tech.rary.gates --version 1.0.0
    orbit install --git https://github.com/c-rus/gates.git -v latest
";