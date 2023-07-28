# __orbit new__

## __NAME__

new - create a new ip

## __SYNOPSIS__

```
orbit new [options] <path>
```

## __DESCRIPTION__

This command will create a new ip at the target directory \<path>. The command
assumes the path does not already exists. It will attempt to create a new 
directory at the destination with a manifest. 

If no name is supplied, then the ip's name defaults to the final path component
of the path argument. Use the name option to provide a custom name.

This command fails if the path already exists. See the 'init' command for
initializing an already existing project into an ip.

## __OPTIONS__

`<path>`  
      The new directory to make

`--name <name>`  
      The ip name to create

## __EXAMPLES__

```
orbit new gates
orbit new ./projects/dir7 --name adder
```

