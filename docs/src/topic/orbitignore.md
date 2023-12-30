# .orbitignore

A _.orbitignore_ is a file that lists user-defined file patterns to ignore during the execution of `orbit`. Orbit recognizes .orbitignore files that match ".orbitignore" with case-sensitivity.

.orbitignore files are typically encouraged to be checked into version control.

## Syntax

.orbitignore files follow the same syntax as .gitignore files. See the pattern format for more information: 
- [.gitignore pattern format](https://git-scm.com/docs/gitignore#_pattern_format)


## Resolving errors

Orbit prevents duplicate primary design units to be identified within certain situations. For example, duplicate design unit names are not allowed within the same project because Orbit cannot resolve ambiguity in which unit is used where.

An error may look like the following:
```
error: Duplicate primary design units identified as 'my_entity'

location 1: rtl/my_entity_file1.vhd:20:1
location 2: rtl/my_entity_file2.vhd:1:1

hint: To resolve this error, either
    1) Rename one of the units to a unique identifier
    2) Add one of the file paths to a .orbitignore file
```

A .orbitignore file can be used in this scenario to tell Orbit to ignore reading a particular file during HDL analysis.

Filename: .orbitignore
```
rtl/my_entity_file2.vhd

```

The .orbitignore will prevent Orbit from seeing the file "rtl/my_entity_file2.vhd" during program execution, which circumvents the previous error.