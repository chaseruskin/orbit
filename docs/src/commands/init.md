# __orbit init__

## __NAME__

init - initialize an ip from an existing project

## __SYNOPSIS__

```
orbit init [options] [<path>]
```

## __DESCRIPTION__

Initializes an ip at the file system directory `<path>`. If not path is
provided, then it defaults to the current working directory. 

If no name is provided, then the resulting ip's name defaults to the 
directory's name. Using the `--name` option allows the ip's name to be 
explicitly set.

To create a new ip from a non-existing directory, see the `new` command.

## __OPTIONS__

`<path>`  
      Directory to initialize

`--name <name>`  
      Set the resulting ip's name

`--lib <lib>`  
      Set the resulting ip's library

## __EXAMPLES__

```
orbit init
orbit init projects/gates
orbit init --name adder
```

