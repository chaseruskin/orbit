# Catalog

As a package manager, Orbit must know what packages are available and where packages are stored on your local file system so that they can be operated on. Orbit stores your projects in the _catalog_. The _catalog_ is a set of directories on your local file system maintained by orbit. These directories are typically hidden from the user because they are not regularly interfacing with the file system contents at these locations and manually tampering the contents may cause trouble for Orbit when it tries to use them.

## Levels

There are three levels to the catalog: the cache, the archive, and channels.

### Cache

The _cache_ maintains the projects that are currently _installed_ on your local file system. Installed projects can be immediately added as a dependency to your current project.

Default location: `$ORBIT_HOME/cache`

### Archive

The _archive_ maintains the projects that are currently _downloaded_ on your local file system. Downloaded projects can be added as a dependency to your current project only after being installed to the cache.

Default location: `$ORBIT_HOME/archive`

### Channels

_Channels_ are user-defined directories to set up as registries to maintain the projects that are currently _available_ to download or install. These projects may require internet to download their contents and then install to your cache.

Default location: `$ORBIT_HOME/channels`

Adding a new channel is as simple as adding a directory to the location where channels are defined.

## Atomicity and synchronization 

Since the catalog is a location on your local file system, it is susceptible to race conditions if multiple processes try to read and write to this location at the same time. To prevent multiple Orbit processes from introducing race conditions, Orbit implements a file locking mechanism on the catalog. The file locking mechanism relies on Rust's standard library locking functions implemented in [`std::fs::File`](https://doc.rust-lang.org/std/fs/struct.File.html). 

With the file locking mechanism in place for the catalog, Orbit processes may block until granted access to the catalog. A message like the following may appear if another Orbit process has currently locked the catalog:

```
info: waiting for file lock on project catalog directory
```

Once the Orbit process holding the lock has finished executing, it will release the lock for another process wishing to gain access to the catalog.