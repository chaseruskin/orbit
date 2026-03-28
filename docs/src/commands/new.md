# __orbit new__

## __NAME__

new - create a new project

## __SYNOPSIS__

```
orbit new [options] <path>
```

## __DESCRIPTION__

Creates a new project at the target directory `<path>`. The path is assumed to 
not already exist. A new directory will be created at the file system 
destination that contains a minimal manifest file.

If no name is supplied, then the project's name defaults to the final directory 
name taken from `<path>`. Using the `--name` option allows this field to be
explicitly set.

The newly created manifest file is intended to be edited by the user. See more
`Orbit.toml` keys and their definitions at:

   https://chaseruskin.github.io/orbit/reference/manifest.html

For initializing an already existing directory into a project, see the `init` 
command.

## __OPTIONS__

`<path>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Directory to create for the project

`--name <name>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the resulting project's name

`--lib <lib>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the resulting project's library

## __EXAMPLES__

```
orbit new gates
orbit new eecs/lab1 --name adder
```

