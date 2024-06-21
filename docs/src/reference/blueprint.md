# Blueprint

The _blueprint_ is a file containing a list of files required for a particular back end. It is the main way `orbit` communicates to a back end system. 

When the blueprint is created, it is saved to the current working IP's build directory.

## Formats

The currently supported formats are:
- [Tab-separated values](#tab-separated-values): `blueprint.tsv`

## Specifications

Each blueprint format may contain different information and store it in a different way. Refer to each specification to see exactly how the data is communicated through their blueprint.

### Tab-separated values

- Advantages
    - Simple and easy to parse for back ends
- Disadvantages
    - Limited information is sent

The file is divided into a series of _rules_, each separated by a newline character (`\n`).

```
RULE
RULE
...
```

A rule contains information about a particular file. Every rule always has 3 components: a fileset, an identifier, and a filepath. Each component in a rule is separated by a tab character (`\t`).

```
FILESET	IDENTIFIER	FILEPATH
```

The _fileset_ is the group name for the file pattern that matched the given rule's file.

The _identifier_ is the library for the IP which the given rule's file belongs to.

The _filepath_ is the absolute file system path to the given rule's file.

#### Examples

``` text
PY-MODEL	work	/Users/chase/projects/lc3b/sim/models/alu_tb.py
VHDL-RTL	work	/Users/chase/projects/lc3b/rtl/const_pkg.vhd
VHDL-RTL	math	/Users/chase/.orbit/cache/base2-1.0.0-aac9159285/pkg/base2.vhd
VHDL-RTL	work	/Users/chase/projects/lc3b/rtl/alu.vhd
VHDL-SIM	work	/Users/chase/projects/lc3b/sim/alu_tb.vhd

```