# Manifest

The `Orbit.toml` file for each ip is called its manifest. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and contains metadata that is needed to build the ip.

> __Note:__ The manifest's file name is "Orbit.toml", with respect to case-sensitivity.

Every manifest file consists of the following sections:

- [[ip]](#the-ip-section) - Defines an ip.
    - [name](#the-name-field) - The name of the ip.
    - [version](#the-version-field) - The version of the ip.
    - [authors](#the-authors-field) - The authors of the ip.
    - [library](#the-library-field) - The HDL library for the design units within the ip.
    - [description](#the-description-field) - A short description of the ip.
    - [keywords](#the-keywords-field) - A list of simple words categorizing the ip.
    - [source](#the-source-field) - The URL for remotely retrieving the ip.
    - [public](#the-public-field) - The list of files to be visible to other ip.
    - [readme](#the-readme-field) - The path to the README file.
    - [[metadata]](#the-metadata-section) - An unchecked section for custom fields.
- [[dependencies]](#the-dependencies-section) - Ip dependencies.
- [[dev-dependencies]](#the-dev-dependencies-section) - Ip dependencies only used for ongoing development.

### The `[ip]` section

The first section in a `Orbit.toml` file is `[ip]`.

``` toml
[ip]
name = "my-ip" # the name of the package
version = "0.1.0" # the current version
```

The only fields required by `orbit` are name and version.

### The `name` field

``` toml
[ip]
name = "my-ip"
# ...
```

### The `version` field

``` toml
[ip]
# ...
version = "0.1.0"
```

### The `authors` field

``` toml
[ip]
# ...
authors = ["Duncan Idaho", "Gurney Halleck"]

```

### The `library` field

``` toml
[ip]
# ...
library = "work"
```

### The `description` field

``` toml
[ip]
# ...
description = "A short description of the ip"
```

### The `keywords` field

``` toml
[ip]
# ...
keywords = ["cpu", "risc"]
```

### The `source` field

``` toml
[ip]
# ...
source = "https://github.com/cdotrus/orbit/archive/refs/tags/1.0.0.zip"
```

### The `public` field

``` toml
[ip]
# ...
public = ["/rtl"]
```

The `public` field can be used to explicitly specify which files are visible to other ip when being when being referenced as a dependency. The list contains glob-style patterns that conform to .gitignore file semantics, and are always compared relative that ip's root directory.

If no `public` field is present, then all files are implicitly specified as visible (public) to other ip when being referenced as a dependency.

``` toml
[ip]
# ...
source = { url = "https://github.com/cdotrus/orbit.git", protocol = "p-git", tag = "1.0.0" }
```

### The `readme` field

``` toml
[ip]
# ...
readme = "README.md"
```

### The `[metadata]` section

Any type of TOML entry is allowed in this section, as Orbit ignores this section.

``` toml
[ip.metadata]
my-field-1 = true
my-field-2 = "hello world"
# ...
```

### The `[dependencies]` section

The `[dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dependencies]
gates = "1.0.0"
uart = "2.3.1"
```

If the ip has no dependencies, the section can be omitted from the manifest. The ips listed in this section will always be included in the build graph.

### The `[dev-dependencies]` section

The `[dev-dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dev-dependencies]
testkit = "1.3.7"
logic-analyzer = "4.8.0"
```

If the ip has no development dependencies, the section can be omitted from the manifest. The ips listed in this section will not be included in the build graph for when this ip is used as a dependency itself.
