# Manifest

The `Orbit.toml` file for each IP is called its manifest. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and contains metadata that is needed to build the IP.

> __Note:__ The manifest's file name must exactly match the spelling "Orbit.toml", including case-sensitivity.

Every manifest file consists of the following sections:

- [[ip]](#the-ip-section) - Defines an IP.
    - [name](#the-name-field) - The name of the IP.
    - [version](#the-version-field) - The version of the IP.
    - [library](#the-library-field) - The HDL library for the design units within the IP.
    - [summary](#the-summary-field) - A short description of the IP.
    - [keywords](#the-keywords-field) - A list of simple words categorizing the IP.
    - [source](#the-source-field) - The URL for remotely retrieving the IP.
- [[dependencies]](#the-dependencies-section) - IP dependencies.
- [[dev-dependencies]](#the-dev-dependencies-section) - IP dependencies only used for ongoing development.

### The `[ip]` section

The first section in a `Orbit.toml` file is `[ip]`.

``` toml
[ip]
name = "lab1" # the name of the package
version = "0.1.0" # the current version
```

The only fields required by `orbit` are name and version.

### The `name` field

### The `version` field

### The `library` field

### The `summary` field

### The `keywords` field

### The `source` field

### The `[dependencies]` section

The `[dependencies]` section is a table of direct dependencies required for the current IP.

``` toml
[dependencies]
gates = "1.0.0"
uart = "2.3.1"
```

If the IP has no dependencies, the section can be omitted from the manifest. The IPs listed in this section will always be included in the build graph.

### The `[dev-dependencies]` section

The `[dev-dependencies]` section is a table of direct dependencies required for the current IP.

``` toml
[dev-dependencies]
testkit = "1.3.7"
logic-analyzer = "4.8.0"
```

If the IP has no development dependencies, the section can be omitted from the manifest. The IPs listed in this section will not be included in the build graph for when this IP is used as a dependency itself.


<!-- 
## Entries

The following is a list of acceptable entries (key/value pairs) recognized by Orbit in manifest files (`Orbit.toml`).

### `ip.name` : _string_
- project name identifier, third component in the PKGID
- required for every manifest

``` toml
[ip]
name = "gates"
# ...
```

### `ip.library` : _string_
- project library identifier, second component in the PKGID
- required for every manifest

``` toml
[ip]
library = "rary"
# ...
```

### `ip.vendor` : _string_
- project vendor/organization identifier, first component in the PKGID
- required for every manifest

``` toml
[ip]
vendor = "ks-tech"
# ...
```

### `ip.version` : _string_
- semver for the project's current status
- required for every manifest

``` toml
[ip]
version = "1.0.0"
# ...
```

### `ip.repository` : _string_
- remote repository git url
- required to launch an ip to a vendor repository

``` toml
[ip]
repository = "https://github.com/kepler-space-tech/gates.git"
# ...
```

### `ip.summary` : _string_
- short description about the ip

``` toml
[ip]
summary = "a collection of low-level logic gates"
# ...
```

### `ip.changelog` : _string_
- relative path to the ip's CHANGELOG
- auto-detects files named "CHANGELOG.md" in ip's directory
``` toml
[ip]
changelog = "CHANGELOG.md"
# ...
```

### `ip.readme` : _string_
- relative path to the ip's README
- auto-detects files named "README.md" in ip's directory
``` toml
[ip]
readme = "README.md"
# ...
```

### `[dependencies]` : _table_
- user-defined additional keys that specify the minimum version of external ip directly used in the current project
- the complete PKGID is entered as a key, while the minimum required version is entered as the value 

``` toml
[dependencies]
ks-tech.rary.memory = "1.2"
ks-tech.util.toolbox = "3.0.4"
```
-->
