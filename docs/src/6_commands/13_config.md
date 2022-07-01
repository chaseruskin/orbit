# __orbit config__

## __NAME__

config - modify configuration values

## __SYNOPSIS__

```
orbit config [options]
```

## __DESCRIPTION__

This command will alter configuration entries in the `config.toml` file. By
default, it will modify the user's config located at $ORBIT_HOME.

To access an entry (key/value pair), use dots (`.`) to delimit between 
intermediate table identifiers and the final key identifier. 
 
The command modifies the document in three steps. The first modification is it
iterates through all `--append` values. Then, it will insert all `--set`
values. Lastly, it will remove all `--unset` entries.

## __OPTIONS__

`--global`  
      Access the settings to the home configuration file
 
`--local`    
      Access the settings to the project configuration file
 
`--append <key>=<value>...`  
      Add a value to a key that stores a list structure
 
`--set <key>=<value>...`  
      Set the key with the value (integer, string, boolean)
 
`--unset <key>...`  
      Remove the key's entry

## __EXAMPLES__

```
orbit config --set core.path="C:/my/projects" --set core.editor="code"
orbit config --append include="/profile/ks-tech"
orbit config --unset env.VIVADO_PATH --global
```