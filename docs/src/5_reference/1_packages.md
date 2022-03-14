# Packages

## Package Identifier

The package identifier (__pkgid__) is a project's unique string of characters following a specification. This specification is similiar to Xilinx's Vendor-Library-Name-Version (VLNV) format. When referencing a pkgid on the command-line, each section of the pkgid is separated with a dot.

### Rules

The following rules apply to each section in the pkgid (vendor, library, and name):

- must begin with an alphabetic character
- can only contain alphanumeric characters, dashes `-`, and underscores `_`

### Specification Grammar

```
PKGID ::= [[<VENDOR>.]<LIBRARY>.]<NAME>[@<VERSION>]
```

If a pkgid can be determined by Orbit without specifying all parts, then unnecessary parts may be omitted. See examples for more information.



### Example Specifications
| pkgid                       | Vendor | Library | Name         | Version
| -                           | -      | -       | -            | -       
uf-ece.rary.gates             | uf-ece | rary    | gates        | #.#.#
Xilinx.DSP.filter             | Xilinx | DSP     | filter       | #.#.#
eel4712c.lab1                 |        | eel4712c| lab1         | #.#.#
Intel.gfx_lib.vga@v1.0.0      | Intel  | gfx_lib | vga          | 1.0.0
lab2@v3.1                     |        |         | lab2         | 3.1.#
AMD.procs.cpu@v2              | AMD    | procs   | cpu          | 2.#.#
Uf.crypto.simon-cipher@latest | Uf     | crypto  | simon-cipher | #.#.#

> __Note__: In this table, a '#' in a version position represents the highest available value for that position. So, version `#.#.#` would refer to the latest available version.
