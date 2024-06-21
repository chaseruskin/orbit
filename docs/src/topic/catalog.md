# Catalog

As a package manager, Orbit must know what ips are available and where ips are stored on your local file system so that they can be operated on. Orbit stores your ips in the _catalog_. The _catalog_ is a set of directories on your local file system maintained by orbit. These directories are typically hidden from the user because they are not regularly interfacing with the file system contents at these locations and manually tampering the contents may cause trouble for Orbit when it tries to use them.

There are three levels to the catalog: the cache, the archive, and channels.

## Cache

The _cache_ maintains the ips that are currently _installed_ on your local file system. Installed ips can be immediately added as a dependency to your current project.

Default location: `$ORBIT_HOME/cache`

## Archive

The _archive_ maintains the ips that are currently _downloaded_ on your local file system. Downloaded ips can be added as a dependency to your current project only after being installed to the cache.

Default location: `$ORBIT_HOME/archive`

## Channels

_Channels_ are user-defined directories to set up as registries to maintain the ips that are currently _available_ to download or install. These ips may require internet to download their contents and then install to your cache.

Default location: `$ORBIT_HOME/channels`

Adding a new channel is as simple as adding a directory to the location where channels are defined.