// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    install - store an immutable reference to an ip

SYNOPSIS
    orbit install [options] <ip>@[version]...

DESCRIPTION
    This command will get move an ip's project folder to the user defined cache.
    By default, the specified version is the 'latest' released version.

OPTIONS
    --path <path>@[version]...  
          Filesystem path to the ip
     
    --git <url>@[version]...  
          Url to git remote repository for the ip

EXAMPLES
    orbit install ks-tech.rary.gates@1.0.0
    orbit install --git https://github.com/c-rus/gates.git@latest
";