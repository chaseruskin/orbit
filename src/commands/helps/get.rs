// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Fetch an hdl entity for code integration.

Usage:
    orbit get [options] <unit>

Args:
    <unit>                  entity identifier

Options:
    --ip <spec>             ip to reference the unit from
    --json                  export the entity information as valid json
    --component, -c         display component declaration
    --signals,   -s         display constant and signal declarations
    --instance,  -i         display instantation
    --architecture, -a      display detected architectures
    --name <identifier>     set the instance's identifier

Use 'orbit help get' to read more about the command.
"#;