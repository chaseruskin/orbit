# __orbit read__

## __NAME__

read - inspect hdl source code files

## __SYNOPSIS__

```
orbit read [options] <unit>
```

## __DESCRIPTION__

This command will open a primary design units source code file for
examination in a text editor. If the unit is from the current ip, then it
will return the direct reference to the file. If the unit comes from 
outside the current ip, then Orbit will create a read-only copy of the 
file into a temporary directory ORBIT_HOME/tmp

## __OPTIONS__

`<unit>`  
      The primary design unit identifier to access
 
`--variant, -v <version>`  
      Version of ip to fetch
 
`--ip, <pkgid>`  
      The IP to search for the unit
 
`--editor <editor>`  
      The text-editor to open the unit's source code file

## __EXAMPLES__

```
orbit read and_gate --ip ks-tech.rary.gates -v 1.0.0
orbit read multiplier --editor code
```