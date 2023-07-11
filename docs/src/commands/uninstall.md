# __orbit uninstall__

## __NAME__

uninstall - remove an ip from the catalog

## __SYNOPSIS__

```
orbit uninstall [options] <ip>
```

## __DESCRIPTION__

This command will delete the project directory of an IP. By default, Orbit
will delete the IP found on the DEV_PATH.

## __OPTIONS__

`--variant, -v <version>`  
      Access the settings to the home configuration file
 
`--force`    
      Remove the ip regardless of conditions

## __EXAMPLES__

```
orbit uninstall kepler.rary.gates -v dev
orbit uninstall kepler.util.toolbox --variant 2.1
```