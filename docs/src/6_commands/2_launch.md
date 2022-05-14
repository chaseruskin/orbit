# __orbit launch__

## __NAME__

launch - release an ip's next version 

## __SYNOPSIS__

```
orbit launch [options]
```

## __DESCRIPTION__

This command will perform a series of checks against the current ip to 
verify it is able to release a new version and the tag the latest git 
commit with the next version number. 
 
By default, it will only perform a dry run of the launch process to verify 
the procedure will run with no errors. To proceed with a launch to tag the
latest commit, include the '--ready' flag.
 
The next version it will release is the one defined in the Orbit.toml 
manifest file. You can also set the next version on the command-line by 
using the '--next \<version>' option. If this option is used, then a new git
commit will be created by Orbit to save the version change it makes to the 
Orbit.toml. To write a custom message for this commit, include the 
'--message \<message>' option.
 
The '--next \<version>' option will go off of the previous version defined
in the Orbit.toml manifest to determine the next increment. 

## __OPTIONS__

`--ready`  
      perform a real run through the launch process
 
`--next <version>`  
      declare the next version or 'major', 'minor', or 'patch' increment
 
`--message <message>`  
      override the default Orbit commit message when using '--next'

## __EXAMPLES__

```
orbit launch --next 1.0.0
orbit launch --next major --ready
```