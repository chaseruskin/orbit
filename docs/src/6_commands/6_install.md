# __orbit install__

## __NAME__

install - store an immutable reference to an ip

## __SYNOPSIS__

```
orbit install [options] <ip>
```

## __DESCRIPTION__

This command will get move an ip's project folder to the user defined cache.
By default, the specified version is the 'latest' released version.

## __OPTIONS__

`--ver, -v <version>`  
      Version to install
 
`--path <path>`  
      Filesystem path to the ip
 
`--git <url>`  
      Url to git remote repository for the ip

## __EXAMPLES__

```
orbit install ks-tech.rary.gates --ver 1.0.0
orbit install --git https://github.com/c-rus/gates.git -v latest
```