# __orbit get__

## __NAME__

get - pull in an hdl entity to use

## __SYNOPSIS__

```
orbit get [options] <unit>
```

## __DESCRIPTION__

This command will add the requested ip as a dependency to the current 
project. It will grab information about the primary design unit to copy and
paste into the current project.
 
If the ip pkgid is omitted, it will assume to search the current working ip
for the requested unit. 
 
If the `--instance` flag is used without the `--component` flag, it will
display the direct instantiation style code (VHDL-93 feature).  
 
By default the ip associated with the target entity is not added under the 
current ip manifest dependency table. The ip can be written with the 
`--add` flag.

## __OPTIONS__

`<unit>`  
      The entity identifier to access
 
`--variant, -v <version>`  
      Version of ip to fetch
 
`--ip, <pkgid>`  
      The IP to search for the unit
 
`--component, -c`  
      Display the component declaration
 
`--signals, -s`  
      Display the corresponding signal declarations
 
`--instance, -i`  
      Display the instance declaration
 
`--info`  
      Display the code file's initial comment header block
 
`--architecture, -a`
      Display a list of available architectures
 
`--add`
      Write the referenced ip to the current Orbit.toml dependency table
 
`--name <identifier>`
      Specific instance identifier

## __EXAMPLES__

```
orbit get nor_gate --ip ks-tech.rary.gates -csi
orbit get alert_unit --ip ks-tech.util.toolbox --add -v 1.0
```