# Versions

Code evolves over time, and versions provide a method for capturing a project's state at a given time stamp.

Orbit uses the _semantic versioning_ scheme for capturing project's state at given time periods. Semantic versioning uses 3 numeric values to signify different levels of change.

```
version ::= <major>.<minor>.<patch>
```

| Level    | Explanation
| -        | -           
| Major    | incompatible API changes        
| Minor    | adding functionality in backward-compatible way
| Patch    | fixing bugs in backward-compatible way          

To learn more about semantic versioning, visit the official [website](https://semver.org). 

Since everyone's stance on what code changes affect which version level may differ, it's important to keep a changelog highlighting differences among versions.

### Rules

- each level may only contain ASCII digits (`0-9`)

All 3 levels must be given a numeric value consisting of only digits separated by a dot (`.`) character. This is considered a _fully qualified_ version.
```
1.0.0
```

In some scenarios a _partially qualified_ version can be accepted. This means one or more of the version's levels are omitted. 
```
1
1.0
```

When given a partially qualified version, Orbit references the maximum version available that satifies the partially qualified version.
```
Available versions: { 1.0.0, 1.2.0, 1.2.1, 1.5.0, 2.1.0 }
1       -> 1.5.0
1.1     -> not found
1.2     -> 1.2.1
2       -> 2.1.0
```
### Example Scenarios

A fully qualified version must be written in every project's manifest file.
``` toml
[ip]
# ...
version = "0.1.0"
# ...
```

To request a particular version (partially or fully qualified) when installing packages, you may append the version number after `@v` to the pkgid.

```
$ orbit install ks-tech.rary.gates@v1.5.4
```

You may also request a particular version (partially or fully qualified) for an entity from an IP .
```
$ orbit get ks-tech.rary.gates::nor_gate --version 1.5.4
```

### Comparing versions

The following pseudocode provides additional help in learning about how versions are compared (selecting a "higher" version).

```
IF major levels are not equal:
    RETURN version with larger major level value.
ELSE IF minor levels are not equal:
    RETURN version with larger minor level value.
ELSE:
    RETURN version with larger patch level value. 
```