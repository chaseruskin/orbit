# __orbit build__

## __NAME__

build - execute a plugin/backend tool flow

## __SYNOPSIS__

```
orbit build [options] <plugin> [--] [args]...
```

## __DESCRIPTION__

This command will call a user-defined command (plugin). A plugin should
typically require a blueprint.tsv to be generated. The command also
should read the data from the blueprint, and then process the data 
(synthesis, simulation, etc.).

## __OPTIONS__

`-- args...`  
    - Arguments to pass to the requested command

## __EXAMPLES__

```
orbit build xsim -- --waves
```