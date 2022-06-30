# __orbit new__

## __NAME__

new - create a new ip

## __SYNOPSIS__

```
orbit new [options] <ip>
```

## __DESCRIPTION__

This command will create a new IP package. The default destination path is
`$ORBIT_DEV_PATH/<vendor>/<library>/<name>`. If the ORBIT_DEV_PATH 
environment variable is not set and `core.path` entry is absent from 
configuration, Orbit will use the directory where the command was invoked as
the base path.

## __OPTIONS__

`--force`  
      Removes the destination directory if it already exists
  
`--path <path>`  
      Specify the destination path
  
`--template <alias>`  
      Specify a configured template to copy

## __EXAMPLES__

```
orbit new ks-tech.rary.gates
orbit new ks-tech.rary.common --template base --path common
```