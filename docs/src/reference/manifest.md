# Manifest

The `Orbit.toml` file for each ip is called its manifest. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the user and contains metadata that is needed to build the ip. The manifest is read by Orbit to help automatically generate the ip's lock file, `Orbit.lock`.

> __Note:__ The manifest's file name is "Orbit.toml", with respect to case-sensitivity.

## Manifest format 

Every manifest file consists of the following sections:

- [[ip]](#the-ip-section) - Defines an ip.
    - [name](#the-name-field) - The name of the ip.
    - [uuid](#the-uuid-field) - The universally unique identifier of the ip.
    - [description](#the-description-field) - A short description of the ip.
    - [version](#the-version-field) - The version of the ip.
    - [authors](#the-authors-field) - The authors of the ip.
    - [library](#the-library-field) - The HDL library for the design units within the ip.
    - [keywords](#the-keywords-field) - A list of simple words categorizing the ip.
    - [source](#the-source-field) - The URL for remotely retrieving the ip.
    - [channels](#the-channels-field) - The channels to update when publishing the ip.
    - [public](#the-public-field) - Files to be visible to other ip.
    - [exclude](#the-exclude-field) - Files to exclude during file discovery.
    - [readme](#the-readme-field) - The path to the README file.
    - [[metadata]](#the-metadata-section) - An unchecked section for custom fields.
- [[dependencies]](#the-dependencies-section) - Ip dependencies.
- [[dev-dependencies]](#the-dev-dependencies-section) - Ip dependencies only used for ongoing development.

### The `[ip]` section

The first section in a `Orbit.toml` file is `[ip]`.

``` toml
[ip]
name = "my-ip" # the name of the package
uuid = "ecj831jmc018hhhgl1d4rzgw8" # the universally unique identifier
```

The only fields required by Orbit are the name and uuid.

### The `name` field

The ip name is an identifier used to refer to the ip. It is used when listed as a dependency in another package, and as the name of the HDL library for all of its source files when the `library` field is omitted. See [names](./names.md) for more information.

``` toml
[ip]
name = "my-ip"
# ...
```

### The `uuid` field

A random string consisting of 25 characters in base36 encoding (a-z0-9). 

This field is used to safeguard against namespace collisions at the ip-level, and should _not_ be manually edited at any point over the lifetime of an ip.

If needing a UUID, obtain a UUID from Orbit by running `orbit init --uuid`.

``` toml
[ip]
# ...
uuid = "ecj831jmc018hhhgl1d4rzgw8"
```

### The `version` field

The current version of the ip.

Versions must have three numeric parts, the major version, the minor version, and the micro version. Each part is separated by a `.` delimiter. An optional label can be added at the end, denoted by a `-`. See [versions](./versions.md) for more information.

This field is optional and will default to `0.0.0`.

``` toml
[ip]
# ...
version = "0.1.0"
```

### The `authors` field

The optional `authors` field lists in an array the people or organizations that are considered the “authors” of the package. The exact meaning is open to interpretation — it may list the original or primary authors, current maintainers, or owners of the package. An optional email address may be included within angled brackets at the end of each author entry.

``` toml
[ip]
# ...
authors = ["Duncan Idaho", "Gurney Halleck <ghalleck@dune.mov>"]

```

### The `library` field

The optional `library` entry is an identifier used to denote the HDL library for all of the source files found within the ip.

``` toml
[ip]
# ...
library = "axi"
```

### The `description` field

The description is a short blurb about the ip. This should be plain text (not Markdown).

``` toml
[ip]
# ...
description = "A short description of the ip"
```

### The `keywords` field

The `keywords` field is an array of strings that describe this package. This can help when searching for the package in the catalog.

``` toml
[ip]
# ...
keywords = ["cpu", "risc"]
```

### The `source` field

The `source` entry is the URL that points to this ip on the internet. The entry's value can either be a string or an inline table.

When the value is a string, it represents the URL. The default protocol will be used on this URL, which is currently curl.

``` toml
[ip]
# ...
source = "https://github.com/chaseruskin/orbit/archive/refs/tags/1.0.0.zip"
```

When the value is an inline table, there are 3 fields that can be specified: the `url`, the `protocol`, and a `tag`. The `url` key is the string for the URL, the `protocol` key is the name of a configured protocol, and the `tag` key is optional metadata that can be used by the specified protocol.

The source field's `url` and `tag` keys support [_string swapping_](./../topic/swapping.md).

``` toml
[ip]
# ...
source = { url = "https://github.com/chaseruskin/orbit.git", protocol = "git", tag = "1.0.0" }
```

### The `channels` field

The `channels` field is an array of strings listing the names of configured channels that this ip will publish to during the publishing process.

Without a channel listed in the `channels` entry, an ip is unable to be published since there is no default channel.

``` toml
[ip]
# ...
channels = ["hyperspace-labs"]
```

### The `public` field

The `public` field is an array of strings that specify which files are publicly accessible to external ips.

``` toml
[ip]
# ...
public = [
    "rtl/",
    "!*_tb.sv"
]
```

The `public` field can be used to explicitly specify which files are visible to other ip when being when being referenced as a dependency. The list contains glob-style file patterns that conform to .gitignore file semantics and are always compared relative to that ip's root directory.

If no `public` field is present, then all files are implicitly specified as invisible (private) to other ip when being referenced as a dependency. To change the default behavior when the `public` field is absent, see the configuration's [require-public](./configuration.md#the-require-public-field) entry.

When mixing un-ignore (`!`) and ignore patterns, the order matters. The latest glob in the list among any overlapping globs will be the one effectively used for that particular pattern.

### The `exclude` field

The `exclude` field is an array of strings that can be used to explicitly specify which files are omitted from file discovery and source code analysis. The patterns specified in the `exclude` field identify a set of files that are not included. The patterns for this field follow .gitignore file semantics, such as using special meanings for `*` and `!` symbols.

``` toml
[ip]
# ...
exclude = [
    "scrap/",
    ".*.vhd"
]
```

The default if this field is not specified is to include all files from the root of the ip, except for the exclusions below. 

Regardless of whether `exclude` is specified, the following files are always excluded:
- Any sub-ips will be skipped (any subdirectory that contains an `Orbit.toml` file)
- Any directories with file named "CACHEDIR.TAG" (this file denotes a target directory)
- Any files ignored by version control (such as those listed in a `.gitignore` file)

The following files are always included:
- The `Orbit.toml` file of the ip itself is always included
- The `Orbit.lock` file of the ip itself is always included

When mixing un-ignore (`!`) and ignore patterns in the `exclude` entry, the order matters. The latest glob in the list among any overlapping globs will be the one effectively used for that particular pattern.

### The `readme` field

The `readme` field should be the path to a file in the ip root (relative to this Orbit.toml) that contains general information about the ip.

``` toml
[ip]
# ...
readme = "README.md"
```

### The `[metadata]` section

Any type of TOML entry is allowed in this section, as Orbit ignores this section. 

``` toml
[ip.metadata]
custom-field-1 = true
custom-field-2 = "hello world"
# ...
```

You can even add additional tables within this section to organize your own project metadata.

``` toml
[ip.metadata.toolchains]
vivado = "2024.1"
# ...

[ip.metadata.vivado]
part = "xc7a35tcpg236–1"
# ...
```

### The `[dependencies]` section

The `[dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dependencies]
gates = "1.0.0"
uart = "2.3.1"
```

If the ip has no dependencies, the section can be omitted from the manifest. The ips listed in this section will always be included in the build graph. For ips that should only be included in the build graph when developing the ip, see the `[dev-dependencies]` section.

Dependencies can also be specified as local when given a file system path that points to the directory where that ip's manifest exists.

``` toml
[dependencies]
spi = { path = "../spi", version = "0.1.2" }
```

Local dependencies are useful when trying to test modifications of that ip within the context of another ip. Local dependencies are also supported in the `[dev-dependencies]` section.

If adding a dependency that is an ambiguous name within the ip catalog, its uuid is required in order to select the correct ip. When the ip's name is unique within the ip catalog, specifying the uuid is optional when listing dependencies.

``` toml
[dependencies]
pcie = { uuid = "3p3oajkuoukigs45fskr0svnv", version = "1.3.0" }
```

Explicitly providing the uuid for a dependent ip is also supported in the `[dev-dependencies]` section.

See [Specifying Dependencies](./../user/specifying_deps.md) for the complete list of ways dependencies can be specified.

### The `[dev-dependencies]` section

The `[dev-dependencies]` section is a table of direct dependencies required for the current ip.

``` toml
[dev-dependencies]
testkit = "1.3.7"
logic-analyzer = "4.8.0"
```

If the ip has no development dependencies, the section can be omitted from the manifest. The ips listed in this section will not be included in the build graph for when this ip is used as a dependency itself.

Refer to the `[dependencies]` section for related capabilities in specifying ips.

See [Specifying Dependencies](./../user/specifying_deps.md) for the complete list of ways dependencies can be specified.