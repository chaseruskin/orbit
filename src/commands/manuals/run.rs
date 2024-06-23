// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    run - prepare and execute a target

SYNOPSIS
    orbit run [options] [--] [args]...

DESCRIPTION
    This command will plan and build for the given target.

OPTIONS
    --top <unit>
        Set the top level design unit

    --bench <unit>
        Set the top level testbench unit

    --fileset <key=glob>...
        A glob-style pattern identified by name to include in the blueprint

    --clean
        Remove all previous files from the target directory before execution

    --force
        Ignore reading the precomputed lock file

    --target <name>
        Target to execute

    --all
        Include all locally found HDL files

    --command <cmd>
        Command to execute

    --list
        View available targets

    --target-dir <dir>
        The relative directory to locate the blueprint file

    --verbose
        Display the command being executed

    args
        Arguments to pass to the target

EXAMPLES
    orbit run --top top --target quartus -- --synth
"#;
