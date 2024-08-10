# File Visibility

An ip's manifest allows for users to set an `exclude` field, which can store a list of user-defined file patterns for Orbit to ignore during file discovery.

## Format

Listing files in the `exclude` field follow the same syntax as .gitignore files. See the pattern format for more information: 
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
    2) adding one of the file paths to the manifest's "ip.exclude" field
```

The `exclude` field can be used in this scenario to tell Orbit to ignore reading a particular file during the HDL source code dependency analysis.

Filename: Orbit.toml
``` toml
[ip]
# ...
exclude = [
    "rtl/foo2.vhd"
]
```

In this example, the value for the above `exclude` field in the local ip's manifest will resolve the previous error because it prevents Orbit from seeing the file "rtl/foo2.vhd" during any file discovery operations.