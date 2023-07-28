// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    config - modify configuration values

SYNOPSIS
    orbit config [options]

DESCRIPTION
    This command will alter configuration entries in Orbit's settings file named
    'config.toml'. By default, it will modify the user's config located at
    $ORBIT_HOME.
      
    To access an entry (key/value pair), use dots ('.') to delimit between 
    intermediate table identifiers and the final key identifier.
      
    The command modifies the document in three independent stages. The first stage
    modifies the settings by iterating through all defined '--append' values. Then, 
    it will insert all '--set' values. Lastly, it will remove all '--unset' entries.

OPTIONS
    --global
        Access the home configuration file

    --local
        Access the current project's configuration file

    --append <key>=<value>...
        Add a value to the key storing a list

    --set <key>=<value>...
        Write the value at the key's entry

    --unset <key>...
        Delete the key's entry

EXAMPLES
    orbit config --append include="~/.orbit/profiles/ks-tech"
    orbit config --unset env.VIVADO_PATH --global
"#;