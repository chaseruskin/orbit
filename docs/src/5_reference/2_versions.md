# Versions

Code evolves over time, and versions provide a method for capturing a project's state at a given time stamp.

Orbit uses the _semantic versioning_ scheme for capturing project's state at given time periods. Semantic versioning uses 3 numeric values to signify different levels of change.

```
<MAJOR>.<MINOR>.<PATCH>
```

| Level    | Explanation
| -        | -           
| Major    | incompatible API changes        
| Minor    | adding functionality in backward-compatible way
| Patch    | fixing bugs in backward-compatible way          

To learn more about semantic versioning, visit the official [website](https://semver.org). 

Since everyone's stance on what code changes affect which version level may differ, it's important to keep a changelog highlighting differences among versions.

## Example Scenarios

The version must be written in every project's manifest file.
```
[ip]
...
version = 0.1.0
...
```

To request a particular version when installing packages, you may append the version number after `@v` to the pkgid.

```
orbit install uf-ece.rary.gates@v1.5.4
```

You may also request a particular version of an entity from an IP.
```
orbit get uf-ece.rary.gates::nor_gate --version 1.5.4
```

## Comparing versions

The following is pseudocode to learn about how versions are compared (selecting 'higher' version).

```
IF major levels are not equal:
    RETURN version with larger major level value.
ELSE IF minor levels are not equal:
    RETURN version with larger minor level value.
ELSE:
    RETURN version with larger patch level value. 
```