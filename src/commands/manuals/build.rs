// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    build - execute a target

SYNOPSIS
    orbit build [options] [--] [args]...

DESCRIPTION
    This command carries out the "building phase". This phase involves running a
    user-defined command or target as a subprocess. It is required that the
    planning phase occurs before the building phase. This command acts upon the 
    current working ip.
    
    If a target was previously used during the planning phase, then this command
    by default will reference and call that target after loading the previously
    written '.env' file from the planning phase. Either a target from '--target' 
    or a command from '--command' is required if a target was not previously
    specified during planning.
    
    If '--list' is used, then it will display a list of the available targets to
    the user. Using '--list' in combination with a target from '--target' will
    display any detailed help information the target has documented in its 
    definition.
    
    As a refresher, a backend workflow typically performs three tasks:  
       1. Parse the blueprint file  
       2. Process the referenced files listed in the blueprint  
       3. Generate an output product 
    
    Any command-line arguments entered after the terminating flag '--' will be
    passed in the received order as arguments to the subprocess's command. If a target already
    has defined arguments, the additional arguments passed from the command-line
    will follow the previously defined arguments.
    
    The subprocess will spawn from the current working ip's build directory.

OPTIONS
    --target <name>
        Target to execute

    --command <cmd>
        Command to execute

    --list
        View available targets

    --force
        Execute the command without checking for a blueprint

    --target-dir <dir>
        The relative directory to locate the blueprint file

    --verbose
        Display the command being executed

    args
        Arguments to pass to the target or command

EXAMPLES
    orbit build --target xsim -- --elab
    orbit build --command python -- synth.py
    orbit build --verbose
    orbit build --target xsim --force -- --help
"#;
