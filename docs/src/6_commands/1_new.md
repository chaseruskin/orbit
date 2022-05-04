# __orbit new__

## __NAME__

new - create a new ip

## __SYNOPSIS__

```
orbit new [options] <ip>
```

## __DESCRIPTION__

This command will create a new IP package. The default destination path is
`$ORBIT_PATH/<vendor>/<library>/<name>`. If the ORBIT_PATH environment
variable is not set nor is the `core.path` key in the config.toml, Orbit
will use the command's relative path to create the corresponding
directories.

## __OPTIONS__

`--force`  
    - Removes the destination directory if it already exists
  
`--path <path>`  
    - Specify the destination path

## __EXAMPLES__

```
orbit new space-tech.rary.gates
```