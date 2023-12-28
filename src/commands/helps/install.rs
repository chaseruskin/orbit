// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Store an immutable reference to an ip.

Usage:
    orbit install [options]

Options:
    <ip>                ip specification to install from catalog
    --url <url>         URL to install the ip from the internet
    --path <path>       ip's local path to install from filesystem
    --protocol <name>   defined protocol to download the package
    --tag <tag>         unique tag to pass to the protocol
    --all               install all dependencies including development
    --list              view available protocols and exit
    --verbose           display the command(s) being executed
    --force             install regardless of cache slot occupancy

Use 'orbit help install' to read more about the command.
"#;