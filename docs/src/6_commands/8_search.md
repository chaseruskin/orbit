# __orbit search__

## __NAME__

search - browse the ip catalog

## __SYNOPSIS__

```
orbit search [options] [<pkgid>]
```

## __DESCRIPTION__

This command will search for all ip defined by Orbit in 3 locations. Use 
the flags to control what areas to search under (`--install`, `--develop`, 
`--available`). An optional pkgid can also be provided to narrow results
even further. Pkgid fields can be omitted by entering an empty value.

## __OPTIONS__

`<pkgid>`  
      Identifiers to filter under V.L.N
 
`--install, -i`  
      Filter for ip installed to the cache
 
`--develop, -d`  
      Filter for ip in-development within the orbit path
 
`--available, -a`  
      Filter for ip available via registries

## __EXAMPLES__

```
orbit search --develop --install --available
orbit search rary. -i
orbit search gates -ia
orbit search ks-tecth.rary.gates -d
```