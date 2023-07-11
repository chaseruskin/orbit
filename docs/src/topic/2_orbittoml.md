# Orbit.toml

`Orbit.toml` is the manifest file that tells Orbit that a folder is an IP. The data is stored in a TOML file format. To learn more about TOML files, visit the TOML [website](https://toml.io/en/). 

> __NOTE:__ The name must preserve case sensitivity and exactly match the spelling "Orbit.toml".

Here is a minimal example manifest:
``` toml
[ip]
vendor  = "ks-tech"
library = "rary"
name    = "gates"
version = "0.1.0"
```

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