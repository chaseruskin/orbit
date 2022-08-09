// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    sync - refresh vendor remotes

SYNOPSIS
    orbit sync [options]

DESCRIPTION
    This command will perform synchronization on all configured vendors that have
    a git remote. To synchronize, the repository will first perform a restore to
    get the repository to a clean state. Then, it will perform a git pull, to 
    be followed by a git push.

OPTIONS
    --vendor <alias>...  
          Access the settings to the home configuration file

EXAMPLES
    orbit sync
    orbit sync --vendor ks-tech --vendor c-rus
";