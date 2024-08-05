# __orbit new__

## __NAME__

new - create a new ip

## __SYNOPSIS__

```
orbit new [options] <path>
```

## __DESCRIPTION__

Creates a new ip at the target directory `<path>`. The path is assumed to not
already exist. A new directory will be created at the file system destination
that contains a minimal manifest and .orbitignore file.

If no name is supplied, then the ip's name defaults to the final directory name
taken from `<path>`. Using the `--name` option allows this field to be
explicitly set.

For initializing an already existing project into an ip, see the `init` 
command.

## __OPTIONS__

`<path>`  
      Directory to create for the ip

`--name <name>`  
      Set the resulting ip's name

`--lib <lib>`  
      Set the resulting ip's library

## __EXAMPLES__

```
orbit new gates
orbit new eecs/lab1 --name adder
```

