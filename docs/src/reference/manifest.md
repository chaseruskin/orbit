# Manifest

The `Orbit.toml` file for each project is called its manifest. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the user and contains metadata that is needed to describe the project. The manifest is read by Orbit to help automatically generate the project's lock file, `Orbit.lock`.

> __Note:__ The manifest's file name is "Orbit.toml", with respect to case-sensitivity.

## Manifest format 

Every manifest file consists of the following sections:

- [[project]](#the-project-section) - Defines a project.
    - [name](#the-name-field) - The name of the project.
    - [uuid](#the-uuid-field) - The universally unique identifier of the project.
    - [version](#the-version-field) - The version of the project.
    - [description](#the-description-field) - A short description of the project.
    - [documentation](#the-documentation-field) - URL of the project documentation.
    - [authors](#the-authors-field) - The authors of the project.
    - [library](#the-library-field) - The HDL library for the design units within the project.
    - [source](#the-source-field) - The URL of the project source repository.
    - [public](#the-public-field) - Source files to be visible to other project.
    - [keywords](#the-keywords-field) - A list of simple words categorizing the project.
    - [channels](#the-channels-field) - The channels to update when publishing the project.
    - [ignore](#the-ignore-field) - Files to ignore during file discovery.
    - [readme](#the-readme-field) - The path to the README file.
    - [license](#the-license-and-license-file-fields) - The project license.
    - [license-file](#the-license-and-license-file-fields) - The path to the text of the license.
    - [[metadata]](#the-metadata-section) - An unchecked section for custom fields.
- [[dependencies]](#the-dependencies-section) - Project dependencies.
- [[dev-dependencies]](#the-dev-dependencies-section) - Project dependencies only used for ongoing development.

### The `[project]` section

The first section in a `Orbit.toml` file is `[project]`.

``` toml
[project]
name = "my-project" # the name of the package
uuid = "ecj831jmc018hhhgl1d4rzgw8" # the universally unique identifier
```

The only fields required by Orbit are the name and uuid.

### The `name` field

The project name is an identifier used to refer to the project. It is used when listed as a dependency in another package, and as the name of the HDL library for all of its source files when the `library` field is omitted. See [Project ID Specification](./project_id_specification.md) for more information.

``` toml
[project]
name = "my-project"
# ...
```

### The `uuid` field

A random string consisting of 25 characters in base36 encoding (a-z0-9). 

This field is used to safeguard against namespace collisions at the project-level, and should _not_ be manually edited at any point over the lifetime of a project.

If needing a UUID, obtain a UUID from Orbit by running `orbit init --uuid`.

``` toml
[project]
# ...
uuid = "ecj831jmc018hhhgl1d4rzgw8"
```

### The `version` field

The current version of the project.

Versions must have three numeric parts, the major version, the minor version, and the micro version. Each part is separated by a `.` delimiter. An optional label can be added at the end, denoted by a `-`. See [versions](./versions.md) for more information.

This field is optional and will default to `0.0.0`.

``` toml
[project]
# ...
version = "0.1.0"
```

### The `description` field

The description is a short blurb about the project. This should be plain text (not Markdown).

``` toml
[project]
# ...
description = "A short description of the project"
```

### The `documentation` field

The documentation field specifies a URL to a website hosting the project’s documentation.

``` toml
[project]
# ...
documentation = "https://chaseruskin.github.io/orbit"
```

### The `authors` field

The optional `authors` field lists in an array the people or organizations that are considered the “authors” of the package. The exact meaning is open to interpretation — it may list the original or primary authors, current maintainers, or owners of the package. An optional email address may be included within angled brackets at the end of each author entry.

``` toml
[project]
# ...
authors = ["Duncan Idaho", "Gurney Halleck <ghalleck@dune.mov>"]

```

### The `library` field

The optional `library` entry is an identifier used to denote the HDL library for all of the source files found within the project.

``` toml
[project]
# ...
library = "axi"
```

### The `source` field

The `source` entry is a string that represents the URL where this project is stored on the internet.

``` toml
[project]
# ...
source = "https://github.com/chaseruskin/orbit/archive/refs/tags/1.0.0.zip"
```

### The `public` field

The `public` field is an array of strings that specify which source files are publicly accessible to external projects.

``` toml
[project]
# ...
public = [
    "rtl/",
    "!*_tb.sv"
]
```

The `public` field can be used to explicitly specify which source files are visible to other projects when being when being referenced as a dependency. The list contains glob-style file patterns that conform to .gitignore file semantics and are always compared relative to that project's root directory.

If no `public` field is present, then all source files are implicitly specified as invisible (private) to other projects when being referenced as a dependency. To change the default behavior when the `public` field is absent, see the configuration's [require-public](./configuration.md#the-require-public-field) entry.

When mixing un-ignore (`!`) and ignore patterns, the order matters. The latest glob in the list among any overlapping globs will be the one effectively used for that particular pattern.

The source key supports [_string swapping_](./../topic/swapping.md).

### The `keywords` field

The `keywords` field is an array of strings that describe this package. This can help when searching for the package in the catalog.

``` toml
[project]
# ...
keywords = ["cpu", "risc"]
```

### The `channels` field

The `channels` field is an array of strings listing the names of configured channels that this project will publish to during the publishing process.

Without a channel listed in the `channels` entry, a project is unable to be published since there is no default channel.

``` toml
[project]
# ...
channels = ["hyperspace-labs"]
```

### The `ignore` field

The `ignore` field is an array of strings that can be used to explicitly specify which files are omitted from file discovery and source code analysis. The patterns specified in the `ignore` field identify a set of files that are not included. The patterns for this field follow .gitignore file semantics, such as using special meanings for `*` and `!` symbols.

``` toml
[project]
# ...
ignore = [
    "scrap/",
    ".*.vhd"
]
```

The default if this field is not specified is to include all files from the root of the project, except for the exclusions below. 

Regardless of whether `ignore` is specified, the following files are always ignored:
- Any sub-projects will be skipped (any subdirectory that contains an `Orbit.toml` file)
- Any directories with file named "CACHEDIR.TAG" (this file denotes a target directory)
- Any files ignored by version control (such as those listed in a `.gitignore` file)

The following files are always included:
- The `Orbit.toml` file of the project itself is always included
- The `Orbit.lock` file of the project itself is always included

When mixing un-ignore (`!`) and ignore patterns in the `ignore` entry, the order matters. The latest glob in the list among any overlapping globs will be the one effectively used for that particular pattern.

### The `readme` field

The `readme` field should be the path to a file in the project root (relative to this Orbit.toml) that contains general information about the project.

``` toml
[project]
# ...
readme = "README.md"
```

### The `license` and `license-file` fields

The `license` field contains the name of the software license that the package is released under. The `license-file` field contains the path to a file containing the text of the license (relative to this Orbit.toml).

Orbit interprets the `license` field as an [SPDX 2.3 license expression](https://spdx.github.io/spdx-spec/v2.3/SPDX-license-expressions/). The name must be a known license from the [SPDX license list 3.26](https://spdx.org/licenses/). See the SPDX site for more information.

SPDX license expressions support AND and OR operators to combine multiple licenses.

``` toml
[project]
# ...
license = "MIT OR Apache-2.0"
```

Using OR indicates the user may choose either license. Using AND indicates the user must comply with both licenses simultaneously. The WITH operator indicates a license with a special exception.

If a project is using a nonstandard license, then the `license-file` field may be specified in lieu of the `license` field.

``` toml
[project]
# ...
license-file = "LICENSE.txt"
```

### The `[metadata]` section

Any type of TOML entry is allowed in this section, as Orbit ignores this section. 

``` toml
[project.metadata]
custom-field-1 = true
custom-field-2 = "hello world"
# ...
```

You can even add additional tables within this section to organize your own project metadata.

``` toml
[project.metadata.toolchains]
vivado = "2024.1"
# ...

[project.metadata.vivado]
part = "xc7a35tcpg236–1"
# ...
```

### The `[dependencies]` section

The `[dependencies]` section is a table of direct dependencies required for the current project.

``` toml
[dependencies]
gates = "1.0.0"
uart = "2.3.1"
```

If the project has no dependencies, the section can be omitted from the manifest. The projects listed in this section will always be included in the build graph. For projects that should only be included in the build graph when developing the current project, see the `[dev-dependencies]` section.

Dependencies can also be specified as local when given a file system path that points to the directory where that project's manifest exists.

``` toml
[dependencies]
spi = { path = "../spi", version = "0.1.2" }
```

Local dependencies are useful when trying to test modifications of that project within the context of another project. Local dependencies are also supported in the `[dev-dependencies]` section.

If adding a dependency that is an ambiguous name within the project catalog, its UUID is required in order to select the correct project. When the project's name is unique within the project catalog, specifying the UUID is optional when listing dependencies.

``` toml
[dependencies]
pcie = { uuid = "3p3oajkuoukigs45fskr0svnv", version = "1.3.0" }
```

Explicitly providing the UUID for a dependent project is also supported in the `[dev-dependencies]` section.

See [Specifying Dependencies](./../user/specifying_deps.md) for the complete list of ways dependencies can be specified.

### The `[dev-dependencies]` section

The `[dev-dependencies]` section is a table of direct dependencies required for the current project.

``` toml
[dev-dependencies]
testkit = "1.3.7"
logic-analyzer = "4.8.0"
```

If the project has no development dependencies, the section can be omitted from the manifest. The projtects listed in this section will not be included in the build graph for when this project is used as a dependency itself.

Refer to the `[dependencies]` section for related capabilities in specifying projects.

See [Specifying Dependencies](./../user/specifying_deps.md) for the complete list of ways dependencies can be specified.