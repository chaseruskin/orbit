// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"View the dependency graph.

Usage:
    orbit tree [options]

Options:
    --root <unit>       uppermost hdl unit to starting the dependency tree
    --compress          replace duplicate branches with a referenced label
    --all               include all possible roots in tree
    --format <fmt>      select how to display unit nodes: 'long' or 'short'
    --ascii             restrict tree chars to the original 128 ascii set
    --ip                view the dependency graph at the ip level

Use 'orbit help tree' to read more about the command.
"#;