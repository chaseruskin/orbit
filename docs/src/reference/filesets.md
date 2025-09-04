# Filesets

A _fileset_ is group of files identified by a common file pattern. Typically they are denoted by a common file extension, such as `.txt`, but a fileset can more broadly be grouped under any glob-style file pattern.

A fileset itself consists of a name and one or more file patterns. 
- The name is a string that is normalized to COBOL-CASE, which uses all upper case letters and the hyphen `-` delimiter. It is used to identify which fileset a file belongs to.
- A file pattern is a glob-style file pattern. It is used to perform glob-style matching when searching the file system for files to add to a fileset.

If a file in the search space matches a least one of defined file patterns of a fileset, then it will be added under that fileset.

## Built-in filesets

There are built-in filesets that Orbit uses that have special rules and work across all projects, including dependencies. The following filesets are currently built-in with Orbit:
- `VHDL`: VHDL files (.vhd, .vhdl)
- `VLOG`: Verilog files (.v, .vl, .vlg)
- `SYSV`: SystemVerilog files (.sv)

## Custom filesets

Custom filesets are filesets that are be defined by the user for a specific target. By default, these filesets are only searched for in the current project and do not extend any of the project's dependencies. Set the `recursive` entry of a fileset true to apply it to all projects in the dependency graph.

If a pattern does not start with an explicit path delimiter (`/`) or relative path indicator (`./`), then Orbit assumes to look for the fileset across every possible subdirectory in the current project by implicitly prepending the pattern with `**/`.

A custom fileset's pattern is glob-style pattern. See [Glob Patterns](./glob_patterns.md) to learn more about the rules for pattern matching.

A custom fileset's patterns support string swapping. See [String Swapping](./../topic/swapping.md) to learn more about how it can be applied to fileset patterns.

## Applying fileset patterns

A fileset's pattern is applied starting from the current project's root directory, the same directory where the `Orbit.toml` file exists.

## Name normalization examples

Despite how the user enters the name of a fileset in a target's configuration, the name will be normalized to COBOL-CASE. The following are some examples of how the normalization process works on a variety of naming conventions.

| User-defined fileset name  | Normalized fileset name (COBOL-CASE) |
| --------- | ------------------ |
| A-SET     | A-SET              |
| B_Set     | B-SET              |
| set_1     | SET-1              |
| SetTwo | SETTWO          |

The normalized fileset name is the name that will be written to the blueprint file when collecting filesets. This design choice is for consistency across targets when reading and parsing the blueprint.

## Custom pattern assumption examples

Unless starting with a path delimiter (`/`), a pattern of a fileset is assumed to be applied across the project's directory and all of its subdirectories. The following are some examples of how Orbit interprets a fileset's patterns.

| User-defined pattern | Interpreted pattern |
| - | - |
| `*.txt` | `**/*.txt` |
| `Boards/*.toml` | `**/Boards/*.toml` |
| `./Boards/*.toml` | `./Boards/*.toml` |
| `/specific/path.log` | `./specific/path.log` |

The custom patterns begin their search for files at the current project's root directory. The file pattern used by Orbit when collecting files for custom filesets is the interpreted pattern.