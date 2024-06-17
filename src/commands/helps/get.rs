// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Fetch an hdl entity for code integration.

Usage:
    orbit get [options] <unit>

Args:
    <unit>                  entity identifier

Options:
    --ip <spec>             ip to reference the unit from
    --json                  export the entity information as valid json
    --library,   -l         display library declaration
    --component, -c         display component declaration
    --signals,   -s         display constant and signal declarations
    --instance,  -i         display instantation
    --architecture, -a      display detected architectures
    --name <identifier>     set the instance's identifier
    --signal-prefix <value> prepend information to the instance's signals
    --signal-suffix <value> append information to the instance's signals

Use 'orbit help get' to read more about the command.
"#;
