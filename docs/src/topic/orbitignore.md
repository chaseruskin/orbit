# .orbitignore

A _.orbitignore_ is a file that lists user-defined file patterns for Orbit to ignore during file discovery. Orbit recognizes .orbitignore files that match ".orbitignore" with case-sensitivity.

.orbitignore files are typically encouraged to be checked into version control.

## Format

.orbitignore files follow the same syntax as .gitignore files. See the pattern format for more information: 
- [.gitignore pattern format](https://git-scm.com/docs/gitignore#_pattern_format)


## Resolving errors

Orbit prevents duplicate primary design units to be identified within certain situations. For example, duplicate design unit names are not allowed within the same project because Orbit cannot resolve ambiguity in which unit is used where.

An error may look like the following:
```
error: duplicate primary design units identified as "foo"

location 1: rtl/foo1.vhd:20:1
location 2: rtl/foo2.vhd:1:1

hint: resolve this error by either
    1) renaming one of the units to a unique identifier
    2) adding one of the file paths to a .orbitignore file
```

A .orbitignore file can be used in this scenario to tell Orbit to ignore reading a particular file during HDL source code dependency resolution.

Filename: .orbitignore
```
rtl/foo2.vhd

```

This example .orbitignore will resolve the previous error because it prevents Orbit from seeing the file "rtl/foo2.vhd" during any file discovery operations.