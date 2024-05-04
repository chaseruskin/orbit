// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Modify configuration values.

Usage:
    orbit config [options]

Options:
    --global                    access the home configuration file
    --local                     access the current project configuration file
    --append <key>=<value>...   add a value to a key storing a list
    --set <key>=<value>...      write the value at the key entry
    --unset <key>...            delete the key's entry

Use 'orbit help config' to read more about the command.
"#;
