// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Fetch packages from the internet.

Usage:
    orbit download [options]

Options:
    --list              print URLs to the console and exit
    --missing           filter only uninstalled packages (default: true)
    --all               include dependencies of all types
    --queue <dir>       set the destination directory to place fetched codebase
    --verbose           display the command being executed
    --force             fallback to default protocol if missing given protocol

Use 'orbit help download' to read more about the command.
"#;