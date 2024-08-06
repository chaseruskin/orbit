# __orbit view__

## __NAME__

view - display metadata of an ip

## __SYNOPSIS__

```
orbit view [options] [<ip>]
```

## __DESCRIPTION__

Displays various bits of information about a particular ip. If no ip is
provided, then it displays information related to the local ip.

To display manifest information, no additional options are required.

To display the defined HDL design elements within the ip, use the `--units`
option. For non-local ip, its protected and private design elements are hidden
from the results. To display design elements of all visibility levels the
`--all` option must also be present.

To display the known versions for an ip, use the `--versions` option.

## __OPTIONS__

`<ip>`  
      Ip spec

`--versions, -v`  
      Display the list of known versions

`--units, -u`  
      Display the hdl design elements defined for this ip

`--all, -a`  
      Include any private or hidden results

## __EXAMPLES__

```
orbit view --units
orbit view gates:1.0.0 -u --all
orbit view gates --versions
orbit view gates:1 -v
```

