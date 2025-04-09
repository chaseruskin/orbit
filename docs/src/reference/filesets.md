# Filesets

A _fileset_ is group of files identified by a common file pattern. Typically they are denoted by a common file extension, such as `.txt`, but a fileset can more broadly be grouped under any glob-style file pattern.

A fileset itself consists of a name and one or more file patterns. 
- The name is a string that is normalized to COBOL-CASE. Cobol case uses all upper case letters with the hyphen `-` delimiter. It is used to identify which fileset a file belongs to.
- A file pattern is a glob-style file pattern. It is used to perform glob-style matching when searching the file system for files to add to a fileset.

If a file in the search space matches a least one of defined file patterns of a fileset, then it will be added under that fileset.

## Built-in filesets

There are built-in filesets that Orbit uses that have special rules and work across all ip, including dependencies. The following filesets are currently built-in with Orbit:
- `VHDL`: VHDL files (.vhd, .vhdl)
- `VLOG`: Verilog files (.v, .vl, .vlg)
- `SYSV`: SystemVerilog files (.sv)

## Custom filesets

Custom filesets are filesets that are be defined by the user for a specific target. These filesets are only searched for in the local ip and do not extend any of the ip's dependencies.

If a pattern does not start with an explicit relative path symbol (`.`), then Orbit assumes to look for the fileset across every possible subdirectory in the local ip by implicitly prepending the pattern with `**/`.

## Apply fileset patterns

A fileset's pattern is applied starting from the current ip's root directory, the same directory where the `Orbit.toml` file exists.

## Name normalization examples

| User-defined Fileset  | Normalized Fileset |
| --------- | ------------------ |
| GOOD-SET  | GOOD-SET           |
| Set-1     | SET-1              |
| set_2     | SET-2              |
| set_three | SET-THREE          |

The normalized fileset name is the name that will be written to the blueprint file when collecting filesets. This design choice is for consistency across targets when reading and parsing the blueprint.

## Custom pattern assumption examples

| User-defined pattern | Interpreted pattern |
| - | - |
| `*.txt` | `**/*.txt` |
| `Boards/*.toml` | `**/Boards/*.toml` |
| `./specific/path.log` | `./specific/path.log` |

The custom patterns begin their search for files at the local ip's root directory. The interpreted pattern is the actual glob-style pattern used when collecting files for custom filesets.