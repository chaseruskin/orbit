# __orbit install__

## __NAME__

install - store an immutable reference to an ip

## __SYNOPSIS__

```
orbit install [options]
```

## __DESCRIPTION__

This command will get move an ip's project folder to the user defined cache.
By default, the specified version is the 'latest' released version orbit can
identify.
  
An ip can be installed from multiple locations. A common method is to
reference the ip with its pkgid if it is already in your ip catalog. Another
method is to install by providing the remote git repository url to clone.
A third method is to provide the local filesystem path to the ip.
  
The version is the "snapshot" of the ip's state during that time of
development. Versions are recognized by git tags following the semver
specification (major.minor.patch).

## __OPTIONS__

`--ip <ip>`  
      PkgID to access an orbit ip to install
 
`--variant, -v <version>`  
      Version to install
 
`--path <path>`  
      Filesystem path to the ip
 
`--git <url>`  
      Url to git remote repository for the ip
 
`--disable-ssh`  
      Convert SSH to HTTPS urls when fetching external dependencies

## __EXAMPLES__

```
orbit install --ip ks-tech.rary.gates --version 1.0.0
orbit install --git https://github.com/c-rus/gates.git -v latest
```