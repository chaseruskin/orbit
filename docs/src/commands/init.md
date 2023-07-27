# __orbit init__

## __NAME__

init - initialize an ip from an existing project

## __SYNOPSIS__

```
orbit init [options] [path]
```

## __DESCRIPTION__

This command will initialize a new ip at the target directory [path]. If no path
is supplied, then it defaults to the current working directory.

If no name is supplied, then the ip's name defaults to the final path component
of the path argument. Use the name option to provide a custom name.

This command fails if the path does not exist. See the 'new' command for
creating an ip from a non-existing directory.

## __OPTIONS__

`[path]`  
      The location to initialize an ip

`--name <name>`  
      The name of the ip

`--force`  
      Overwrite a manifest if one already exists

## __EXAMPLES__

```
orbit init
orbit init ./projects/lab1
orbit init --name hello_world
```

