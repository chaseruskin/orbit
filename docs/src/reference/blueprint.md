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
`VLOG` | .v, .vl, .verilog, .vlg, .vh |
`SYSV` | .sv, .svh |

## Tab-separated values

- Advantages
    - Simple and easy to parse for back ends
- Disadvantages
    - Limited information is sent

The file is divided into a series of _steps_, each separated by a newline character (`\n`).

```
STEP
STEP
...
```

A step contains information about a particular file. Every step always has 3 components: a fileset, a library, and a filepath. Each component in a step is separated by a tab character (`\t`).

```
FILESET	LIBRARY	FILEPATH
```

#### Examples

``` text
PYMDL	lc3b	/Users/chase/projects/lc3b/sim/models/alu_tb.py
VHDL	lc3b	/Users/chase/projects/lc3b/rtl/const_pkg.vhd
VHDL	base2	/Users/chase/.orbit/cache/base2-1.0.0-aac9159285/pkg/base2.vhd
VHDL	lc3b	/Users/chase/projects/lc3b/rtl/alu.vhd
VHDL	lc3b	/Users/chase/projects/lc3b/sim/alu_tb.vhd
```