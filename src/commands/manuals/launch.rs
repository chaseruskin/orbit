// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    launch - release an ip through a channel

SYNOPSIS
    orbit launch [options]

DESCRIPTION
    Performs a series of checks on a stable version of a local ip to then release it
    through a channel.
    
    For an ip to be launched, it must have its source field defined that directs to
    a valid internet location.
    
    By default, it operates a dry run, performing all steps in the process except
    for the actual release through the channel. To fully run the command, use the
    '--ready' flag. When the ip is launched, it will also be installed to the cache
    by default. To skip this behavior, use the '--no-install' flag.

OPTIONS
    --ready
        Perform a full run

    --no-install
        Skip installing the ip

    --list
        View available channels and exit

EXAMPLES
    orbit launch
    orbit launch --ready
"#;
