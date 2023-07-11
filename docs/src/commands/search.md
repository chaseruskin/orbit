# __orbit search__

## __NAME__

search - browse the ip catalog

## __SYNOPSIS__

```
orbit search [options] [<pkgid>]
```

## __DESCRIPTION__

This command will search for all ip defined by Orbit in the catalog from all
3 state: development, installation, and available. You can control what 
states to search for with `--develop`, `--install`, and `--available` flags.
  
An optional pkgid can also be provided to narrow results even further. Pkgid 
fields can be omitted by entering an empty value.

## __OPTIONS__

`<pkgid>`  
      Identifiers to filter under vendory.library.name
 
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