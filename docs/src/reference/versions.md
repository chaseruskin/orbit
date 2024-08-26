# Versions

Code evolves over time, and versions provide a method for capturing a project's state at a given time stamp.

Orbit uses the _semantic versioning_ scheme for capturing project's state at given time periods. Semantic versioning uses 3 numeric values to signify different levels of change.

```
version ::= major "." minor "." micro [ "-" label ]
```

| Level    | Explanation
| -        | -           
| Major    | Incompatible API changes        
| Minor    | Adding functionality in backward-compatible way
| Micro    | Fixing bugs in backward-compatible way
| Label    | Descriptive modifier to show status of upcoming version      

To learn more about semantic versioning, visit the official [website](https://semver.org). 

Determining the next version number based on a project's recent code changes can be an opinionated process, so it's recommended to also keep a changelog highlighting the differences among versions.

> __Note:__ An alternative to _semantic versioning_ is _calender versioning_, which
also operates on the basis of using 3 digits. To learn more about _calender versioning_ visit the official [website](https://calver.org).

## Rules

- Each level may only contain ASCII digits (`0-9`).
- A label must follow a dash character (`-`) and cannot be empty.
- Labels can consist of ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), and/or decimal characters (`.`)

All 3 levels must be given a numeric value consisting of only digits separated by a dot (`.`) character. This is considered a _fully qualified_ version.
```
1.0.0
```

In some scenarios a _partially qualified_ version can be accepted. This means one or more of the version's levels are omitted. A label is not required for a version to be considered fully qualified.
```
1
1.0
```

When given a partially qualified version, Orbit references the maximum version available that satifies the partially qualified version. If no version is specified, it assumes the request is for the latest known version. The latest known version can also be explicitly requested by inputting `latest` as the version. Assume the known released versions for a given IP are as listed: 

Versions | 
---------|
`2.1.0`    |
`1.5.0`    |
`1.2.1`   |
`1.2.0`    |
`1.0.0`    |

The following illustrates the mapping from the partially specified requested version to its fully specified known version that would be returned:

Requested | Returned  |
----------|-----------|
`1`        | `1.5.0`     |
`1.1`       | `NOT FOUND` |
`1.2`       | `1.2.1`     |
`2`         | `2.1.0`     |
`1.2.0`     | `1.2.0`     |
`latest`    | `2.1.0`     |
`(omitted)` | `2.1.0`     |

## Example

A fully qualified version must be written in every project's manifest file.

``` toml
[ip]
# ...
version = "1.5.4"
# ...
```

A specific (or partially speific) version can be requested for an IP on the command-line by placing a colon `:` character between the package's name and the requested version.

```
$ orbit install gates:1.5.4
$ orbit get nor_gate --ip gates:1.5
```

## Comparing versions

The following pseudocode provides additional help in learning about how versions are compared (selecting a "higher" version).

```
IF major levels are not equal:
    RETURN version with larger major level value.
ELSE IF minor levels are not equal:
    RETURN version with larger minor level value.
ELSE:
    RETURN version with larger patch level value. 
```