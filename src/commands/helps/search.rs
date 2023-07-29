// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Browse the ip catalog.

Usage:
    orbit search [options] [<ip>]

Args:
    <ip>                filter the name of ip

Options:
    --install, -i       filter ip installed to cache
    --download, -d      filter ip downloaded to downloads
    --keyword <term>... special word to filter out packages
    --limit <num>       maximum number of results to return
    --match             only return results with each filter passed

Use 'orbit help search' to read more about the command.
"#;