// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    init - initialize an ip from an existing project

SYNOPSIS
    orbit init [options] [path]

DESCRIPTION
    This command will initialize a new ip at the target directory [path]. If no path
    is supplied, then it defaults to the current working directory.
    
    If no name is supplied, then the ip's name defaults to the final path component
    of the path argument. Use the name option to provide a custom name.
    
    This command fails if the path does not exist. See the 'new' command for
    creating an ip from a non-existing directory.

OPTIONS
    [path]
        The location to initialize an ip

    --name <name>
        The name of the ip

    --force
        Overwrite a manifest if one already exists

EXAMPLES
    orbit init
    orbit init ./projects/lab1
    orbit init --name hello_world
"#;