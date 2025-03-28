# Blueprint

The _blueprint_ is a file containing a list of files required for a particular back end. This single file is the main method Orbit uses to communicate information to a target's process.

When the blueprint is created, it is saved to the local ip's target output directory.

## Formats

The currently supported formats are:
- [Tab-separated values](#tab-separated-values): `blueprint.tsv`

## Specifications

Each blueprint format may contain different information and store it in a different way. Refer to each specification to see exactly how the data is communicated through their blueprint.

Attributes that are consistent across all formats are the fileset, library, and filepath.

The _fileset_ is the group name for the file pattern that matched the given rule's file.

The _library_ is the hdl defined library for the ip which the given file at this particular step was found.

The _filepath_ is the absolute file system path to the given rule's file.

### Built-in Filesets

The following filesets are already recognized by Orbit and are used for identifying hdl source code:

Fileset| Supported file extensions |        
-------|---------|    
`VHDL` | .vhd, .vhdl |   
`VLOG` | .v, .vl, .vlg |
`SYSV` | .sv |

## Tab-separated values

- Advantages
    - Simple and easy to parse for backends
- Disadvantages
    - Limited information is sent

The file is divided into a series of ordered _entries_, each separated by a newline character (`\n`).

```
ENTRY
ENTRY
...
```

Each entry contains information about a source file. Every entry always has 3 components: a fileset, a library, and a filepath. Each component in an entry is separated by a tab character (`\t`).

```
FILESET	LIBRARY	FILEPATH
```

#### Examples

The following text represents what a blueprint that is formatted as tab-separated values might look like:

``` text
PYMDL	lc3b	/Users/chase/projects/lc3b/sim/models/alu_tb.py
VHDL	lc3b	/Users/chase/projects/lc3b/rtl/const_pkg.vhd
VHDL	mmry	/Users/chase/.orbit/cache/mmry-1.0.0-aac9159285/src/ram.vhd
VHDL	lc3b	/Users/chase/projects/lc3b/rtl/alu.vhd
VHDL	lc3b	/Users/chase/projects/lc3b/sim/alu_tb.vhd
```