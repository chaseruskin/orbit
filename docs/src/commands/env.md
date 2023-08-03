# __orbit env__

## __NAME__

env - print orbit environment information

## __SYNOPSIS__

```
orbit env [options]
```

## __DESCRIPTION__

This command prints environment variables relevant to `orbit`.

By default, this command prins information as a shell script. If one or more
variable names are given as arguments as `<key>`, then it will print the value
of each provided variables on its own line.

## __OPTIONS__

`<key>...`  
      Include this variable's value specifically in the environment information

## __EXAMPLES__

```
orbit env
orbit env ORBIT_HOME
orbit env ORBIT_CACHE ORBIT_DOWNLOADS
```

