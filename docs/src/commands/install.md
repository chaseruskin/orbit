# __orbit install__

## __NAME__

install - store an immutable reference to a project

## __SYNOPSIS__

```
orbit install [options]
```

## __DESCRIPTION__

This command will place a project into the cache. By default, the specified 
version is the 'latest' released version of the project that Orbit has 
identified.

When this command is ran without specifying the `<project>` or a source (such 
as `--url` or `--path`), it will attempt to install the current project, if it
exists.

By default, any dependencies required only for development by the target
project are omitted from installation. To also install these dependencies, 
use the `--all-deps` flag.

If a protocol is recognized using `--protocol`, then an optional tag can also 
be supplied to help the protocol with providing any additional information it
may require.

The `--path` option can accept a file system path that is either 1) the root 
directory that contains the manifest file or 2) a zip archive file that when 
uncompressed, has the manifest file at the root directoy.

The `--all-public` flag can be used to skip providing the "project.public" 
field in the current project's manifest, granting the assumption that all 
source files should be public. It can only be present when installing from a 
local project and when the "project.public" field does not exist.

To remove a project from the catalog, see the `remove` command.

## __OPTIONS__

`<project>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Project id specification

`--url <url>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Url to install the project from the internet

`--path <path>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Path to install the project from local file system

`--protocol, -p <name>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Protocol to download the project

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Install the project regardless of the cache slot occupancy

`--offline`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Skip checking coherency with source

`--list, -l`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; View available protocols and exit

`--all-deps`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Install all dependencies (including development)

`--all-public`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Install with all source files being public

## __EXAMPLES__

```
orbit install
orbit install lcd_driver:2.0
orbit install adder:1.0.0 --url https://my.adder/project.zip
orbit install alu:2.3.7 --path ./projects/alu --force
```

