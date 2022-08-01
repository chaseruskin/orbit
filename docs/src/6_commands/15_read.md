# __orbit read__

## __NAME__

read - inspect hdl source code files

## __SYNOPSIS__

```
orbit read [options] <entity-path>
```

## __DESCRIPTION__

This command will ...

## __OPTIONS__

`<entity-path>`  
      The pkgid and entity identifier to request [pkgid:]<entity>
 
`--variant, -v <version>`  
      Version of ip to fetch
 
`--editor <editor>`
      The command to open the requested text-editor

## __EXAMPLES__

```
orbit read kepler.rary.gates:and_gate -v 1.0.0
orbit read :multiplier --editor code
```

orbit get multiplier --ip kepler.rary.gate@v1.0.0


orbit get <unit> --ip kepler.rary.gates