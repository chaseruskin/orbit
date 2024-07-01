// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    test - run a test

SYNOPSIS
    orbit test [options] [--] [args]...

DESCRIPTION
    This command prepares a given target and then executes the target.
    
    While this command functions the same as 'orbit build', the targets that are 
    encouraged to be used with this command are ones that are designed to either
    "pass" or "fail", typically through a return code.
    
    A target must be provided for the test command to run. A default target can
    be specified in a configuration file, which will be used when a target is
    omitted from the command-line.
    
    If '--list' is used, then it will display a list of the available targets to
    the user. Using '--list' in combination with a target from '--target' will
    display any detailed help information the target has documented in its 
    definition.
    
    A target typically goes through three steps for the testing process:  
       1. Parse the blueprint file  
       2. Process the referenced files listed in the blueprint  
       3. Verify the hdl source code passes all tests
    
    Any command-line arguments entered after the terminating flag '--' will be
    passed in the received order as arguments to the subprocess's command. If a 
    target already has defined arguments, the additional arguments passed from the 
    command-line will follow the previously defined arguments.
    
    The target's process will spawn from the current working ip's output directory,
    which is $ORBIT_TARGET_DIR/$ORBIT_TARGET.

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
    orbit test --top top --target modelsim -- --lint
"#;
