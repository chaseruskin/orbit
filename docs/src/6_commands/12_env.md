# __orbit env__

## __NAME__

env - print orbit environment information

## __SYNOPSIS__

```
orbit env [options]
```

## __DESCRIPTION__

This command will print environment information set by Orbit during runtime.
By default, it will display all known information in the current 
environment. 
  
Optionally passing in keys will print the value's back in the
order they were accepted on the command line. If a variable does not exist,
it will print an empty line.

## __OPTIONS__

`<key>...`  
      Environment variable keys to request to print

## __EXAMPLES__

```
orbit env
orbit env ORBIT_HOME
orbit env ORBIT_DEV_PATH ORBIT_HOME
```