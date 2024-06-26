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

    --top <unit>
        Set the top level design unit

    --bench <unit>
        Set the top level testbench unit

    --target-dir <dir>
        The relative directory where the target starts

    --command <path>
        Overwrite the target's command

    --list
        View available targets and exit

    --all
        Include all hdl files of the working ip

    --fileset <key=glob>...
        A glob-style pattern identified by name to include in the blueprint

    --force
        Force the target to execute 

    --verbose
        Display the command being executed

    args
        Arguments to pass to the target

EXAMPLES
    orbit build --target xsim -- --elab
    orbit build --command python3 --target pysim
    orbit build --all --target-dir build --target ghdl
    orbit build --target xsim --force -- --help
"#;
