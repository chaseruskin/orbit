# __orbit probe__

## __NAME__

probe - access information about an orbit IP

## __SYNOPSIS__

```
orbit probe [options] <ip>
```

## __DESCRIPTION__

This command will display details for a particular IP. By default, it will
return the `Orbit.toml` manifest file contents.
  
Since IP can exist at 3 different levels, the default IP manifest to return
data about is the latest installed version. If there is none, it will try
the latest available version, and then the development version, if those 
exist. 
  
The `--version` option can accept a partial or full verion value, 'latest', 
or 'dev'. 'latest' will point to the user's highest known version,
and 'dev' will point to the IP in the DEV_PATH.

## __OPTIONS__

`<ip>`  
      The fully specified pkgid for the ip
 
`--tags`  
      Return a list of versions and where they are located
 
`--ver, -v <version>`  
      Extract data from a particular version
 
`--units`  
      List the available primary design units within the IP

## __EXAMPLES__

```
orbit probe ks-tech.rary.gates --tags
orbit probe util.toolbox -v 1.2.3 --units
```