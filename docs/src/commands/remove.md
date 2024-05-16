# __orbit remove__

## __NAME__

remove - uninstall an ip from the catalog

## __SYNOPSIS__

```
orbit remove [options] <ip>
```

## __DESCRIPTION__

This command will remove known ip stored in the catalog. By default, it will
remove the ip from the cache. This include any dynamic entries spawned from the
requested ip to remove.

To remove the ip from the cache and downloads locations, use `--all`.

## __OPTIONS__

`<ip>`  
      Ip specification

`--all`  
      remove the ip from the cache and downloads

`--recurse`  
      fully remove the ip and its dependencies

## __EXAMPLES__

```
orbit remove gates
orbit remove gates:1.0.0 --all
```

