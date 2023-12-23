// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Execute a backend workflow.

Usage:
    orbit build [options] [--] [args]...

Options:
    --plugin <name>    plugin to execute
    --command <cmd>     command to execute
    --list              view available plugins
    --build-dir <dir>   set the output build directory
    --verbose           display the command being executed
    args                arguments to pass to the requested command

Use 'orbit help build' to read more about the command.
"#;