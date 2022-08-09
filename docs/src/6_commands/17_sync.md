# __orbit sync__

## __NAME__

sync - refresh vendor remotes

## __SYNOPSIS__

```
orbit sync [options]
```

## __DESCRIPTION__

This command will perform synchronization on all configured vendors that have
a git remote. To synchronize, the repository will first perform a restore to
get the repository to a clean state. Then, it will perform a `git pull`, to 
be followed by a `git push`.

## __OPTIONS__

`--vendor <alias>...`  
      Access the settings to the home configuration file

## __EXAMPLES__

```
orbit sync
orbit sync --vendor ks-tech --vendor c-rus
```