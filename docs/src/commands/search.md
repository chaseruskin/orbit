# __orbit search__

## __NAME__

search - browse the ip catalog

## __SYNOPSIS__

```
orbit search [options] [<ip>]
```

## __DESCRIPTION__

This command will display a list of all the known ip in the catalog. The catalog
consists of 3 levels: cache, downloads, and channels.

Any ip at the cache level are considered installed. Any ip at the downloads
level are considered downloaded. Any ip at the channels level is considered
available. An ip does not exist in the catalog if it is not found at any one
of the three defined levels.

When a package name is provided for `<ip>`, it will begin to partially match 
the name with the names of the known ip. If an ip's name begins with `<ip>`, it
is included in the filtered resultes. To strictly match the argument against an
ip name, use `--match`.

## __OPTIONS__

`<ip>`  
      The beginning of a package name

`--install, -i`  
      Filter ip installed to the cache

`--download, -d`  
      Filter ip downloaded to the downloads

`--keyword <term>...`  
      Include ip that contain this keyword

`--limit <num>`  
      The maximum number of results to return

`--match`  
      Return results that only pass each filter

## __EXAMPLES__

```
orbit search axi
orbit search --keyword memory --keyword ecc
orbit search --keyword RF --limit 20
```

