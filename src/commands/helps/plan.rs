// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Prepare a target.

Usage:
    orbit plan [options]              

Options:
    --top <unit>            override auto-detected toplevel entity
    --bench <unit>            override auto-detected toplevel testbench
    --target <name>        collect filesets defined for a target
    --target-dir <dir>       set the output build directory
    --fileset <key=glob>... set an additional fileset
    --clean                 remove all files from the build directory
    --list                  view available targets and exit
    --lock-only             create the lockfile and exit
    --all                   include all found HDL files
    --force                 skip reading from the lock file

Use 'orbit help plan' to read more about the command.
"#;
