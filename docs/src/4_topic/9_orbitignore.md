# .orbitignore

Sometimes copies of files will exist within a given project. In this case, you may have duplicate primary design unit identifiers. Having duplicate primary design unit identifiers within the same project results in an error because Orbit cannot resolve ambiguity in which unit to select (as backend tools cannot either).

> __Note:__ `.orbitignore` files must exactly match its case-sensitive spelling for Orbit to detect it.

## Syntax

`.orbitignore` files follow the same syntax as .gitignore files. See the pattern format for more information: [.gitignore pattern format](https://git-scm.com/docs/gitignore#_pattern_format).


## Resolving errors

An error may look like the following:
```
error: duplicate primary design units identified as 'my_entity'

location 1: rtl/my_entity_file1.vhd:20:1
location 2: rtl/my_entity_file2.vhd:1:1

hint: To resolve this error either
    1) rename one of the units to a unique identifier
    2) add one of the file paths to a .orbitignore file
```

A .orbitignore file can be used to tell Orbit to ignore reading that file during HDL analysis. When creating a `.orbitignore` file, it should generally be placed alongside the `Orbit.toml` file.

After reviewing each unit, we decide they are identical and one can be safely ignored for now. We create a file called `.orbitignore` at the root of our current project.

_.orbitignore:_
```
rtl/my_entity_file2.vhd

```

Running commands in Orbit no longer results in an error and Orbit chooses to read the 'my_entity' design unit from `rtl/my_entity_file1.vhd`.

> __Note:__ If using a `.orbitignore` file, it is recommended to check in to version control.
