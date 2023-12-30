# Filesets

A _fileset_ is group of files identified by a common file pattern. Typically they are denoted by a common file extension, such as `.txt`, but a fileset can more broadly be grouped under any glob-style pattern.

A fileset itself consists of a name and a pattern. 
- The name is a string that is normalized to ALL-UPPERCASE-WITH-HYPENS. It is used to identify which fileset a file belongs to.
- The pattern is a glob-style pattern. It is used to perform glob-style matching when searching the file system for files to add to a fileset.

## Built-in filesets

There are built-in filesets that `orbit` uses that have special rules and work across all IP, including dependencies. The filesets built into `orbit` that are currently supported are:
- `VHDL-RTL`: VHDL files (.vhd, .vhdl) that contain zero entities without a port interface. 
- `VHDL-SIM`: VHDL files (.vhd, .vhdl) that contain at least one entity without a port interface.

## Reserved filesets

Filesets that are planned to be built into `orbit` at a later date are:
- `VLOG-RTL`: SytemVerilog/Verilog files (.v, .sv) that contain zero modules without a port interface.
- `VLOG-SIM`: SystemVerilog/Verilog files (.v, .sv) that contain at least one module without a port interface.

## Custom filesets

Custom filesets are filesets that are be defined by the user either for a specific plugin or on the command-line. These filesets are only searched for in the current working IP and do not extend to its dependencies.

If the pattern does not start with an explicit relative path symbol (`.`), then `orbit` assumes to look for the fileset across every possible path in the current working IP by implicitly prepending the pattern with `**/`.


## Name normalization examples

| User-defined Fileset  | Normalized Fileset |
| --------- | ------------------ |
| GOOD-SET  | GOOD-SET           |
| Set-1     | SET-1              |
| set_2     | SET-2              |
| set_three | SET-THREE          |

The normalized fileset name is the name that will be written to the blueprint file when collecting filesets. This design choice is for consistency across plugins when reading and parsing the blueprint.

## Custom pattern assumption examples

| User-defined pattern | Interpreted pattern |
| - | - |
| *.txt | **/*.txt |
| Boards/*.toml | **/Boards/*.toml |
| ./specific/path.log | ./specific/path.log |

The custom patterns begin their search for files at the root directory of the current working IP. The interpreted pattern is the actual glob-style pattern used when collecting files for custom filesets.