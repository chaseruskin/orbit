# __orbit search__

## __NAME__

search - browse the ip catalog

## __SYNOPSIS__

```
orbit search [options]
```

## __DESCRIPTION__

This command will search for all ip defined by Orbit in 3 locations.

## __OPTIONS__

`--cache, -c`  
      Filter for ip installed to the cache
 
`--develop, -d`  
      Filter for ip in-development within the orbit path
 
`--available, -a`  
      Filter for ip available via registries

## __EXAMPLES__

```
orbit search --develop --cache --available
```