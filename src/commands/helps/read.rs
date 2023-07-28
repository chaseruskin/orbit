// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Inspect hdl design unit source code.

Usage:
    orbit read [options] <unit>

Args:
    <unit>                  primary design unit identifier

Options:            
    --ip <spec>             ip to reference the unit from
    --location              append the :line:col to the filepath
    --file                  display the path to the read-only source code
    --keep                  prevent previous files read from being deleted
    --limit <num>           set a maximum number of lines to print
    --start <code>          tokens to begin reading contents from file
    --end <code>            tokens to end reading contents from file
    --doc <code>            series of tokens to find immediate comments for

Use 'orbit help read' to read more about the command.
"#;