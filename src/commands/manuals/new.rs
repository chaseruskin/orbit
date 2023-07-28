// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    new - create a new ip

SYNOPSIS
    orbit new [options] <path>

DESCRIPTION
    This command will create a new ip at the target directory '<path>'. The command
    assumes the path does not already exists. It will attempt to create a new 
    directory at the destination with a manifest. 
    
    If no name is supplied, then the ip's name defaults to the final path component
    of the path argument. Use the name option to provide a custom name.
    
    This command fails if the path already exists. See the 'init' command for
    initializing an already existing project into an ip.

OPTIONS
    <path>
        The new directory to make

    --name <name>
        The ip name to create

EXAMPLES
    orbit new gates
    orbit new ./projects/dir7 --name adder
"#;