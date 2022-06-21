# __orbit build__

## __NAME__

build - execute a plugin/backend tool flow

## __SYNOPSIS__

```
orbit build [options] [--] [args]...
```

## __DESCRIPTION__

This command will call a user-defined command or plugin. A plugin should
typically require a blueprint.tsv to be generated. The command also
should read the data from the blueprint, and then process that data
(synthesis, simulation, etc.).

## __OPTIONS__

`--plugin <alias>`  
      Plugin to execute
 
`--command <cmd>`     
      Command to execute
 
`-- args...`  
      Arguments to pass to the requested plugin

## __EXAMPLES__

```
orbit build xsim -- --waves
```