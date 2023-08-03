# __orbit install__

## __NAME__

install - store an immutable reference to an ip

## __SYNOPSIS__

```
orbit install [options]
```

## __DESCRIPTION__

This command will get move an ip's project folder to the cache. By default,
the specified version is the 'latest' released version orbit can
identify.
  
An ip can be installed from multiple locations. A common method is to
reference the ip with its pkgid if it is already in your ip catalog with
`--ip`. Another method is to install by providing the remote git repository 
url to clone with `--git`. A third method is to provide the local filesystem
path to the ip with `--path`.
  
The version is the "snapshot" of the ip's state during that time of
development. Versions are recognized by Orbit as git tags following the 
semver specification (major.minor.patch).
  
Development versions are not allowed to be installed to the cache
because they are considered mutable.

## __OPTIONS__

`--path <path>`  
      Directory to install ip from to place in the cache

`--ip <name>`  
      Ip to install from the queue into the cache

`--force`  
      Install the ip regardless of the cache slot occupancy

`--all`  
      Install all dependencies (including development)

## __EXAMPLES__

```
orbit install"
orbit install --path ./projects/ram --force 
orbit install --all
```

