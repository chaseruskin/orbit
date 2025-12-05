# Configuration

The `config.toml` file stores settings and extends Orbit's functionality. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and can be shared across teams for consistent development environments.

> __Note:__ The configuration's file name is "config.toml", with respect to case-sensitivity.

## Config-relative paths

Paths in config files may be absolute, relative, or a bare name without any path separators. Paths for executables without a path separator will use the `PATH` environment variable to search for the executable. Paths for non-executables will be relative to where the config value is defined.

In particular, the rules are:
- For environment variables, paths are relative to the current working directory.
- For config values loaded using `orbit config`, paths are left unresolved and stored as-is in the config file.
- For config files, paths are relative to the parent directory of the directory where the config files were defined.

This design choice was implemented to allow path definitions to be valid across file systems when sharing configurations. It is recommended to use relative paths when setting a field's value as a path in a `config.toml`.

## Executable paths with arguments

Some Orbit commands invoke external programs, which can be configured as a path and some number of arguments.

The value may be an array of strings like `['/path/to/program', 'somearg']` or a space-separated string like `'/path/to/program somearg'`. If the path to the executable contains a space, the list form must be used.

If Orbit is passing other arguments to the program such as a path to open or run, they will be passed after the last specified argument in the value of an option of this format. If the specified program does not have path separators, Orbit will search `PATH` for its executable.

## Hierarchical structure

Orbit supports multiple levels of configuration. Each level has its own order of precedence. The order of precedence is the following:

1. Local configuration file (current project's `.orbit/config.toml`)

2. Regional configuration files (parent directories of the current working directory)

3. Global configuration file (Orbit's `$ORBIT_HOME/config.toml`)

4. Included configuration files (order-preserving list found for the global configuration's [`include`](#the-include-field) field)

The configuration files are processed in the order defined above. When a configuration file defines a field, no other configuration files later in the process will be able to override its value. If a field is never provided an explicit value, then Orbit's default will be used.

> __Tip:__ You can modify some values in the configuration file through the command-line by using the `orbit config` command.

## Configuration format

Every configuration file consists of the following sections:

- [include](#the-include-field) - Lists other `config.toml` files to process. This field is only allowed for the global configuration file.
- [[general]](#the-general-section) - The general settings.
    - [require-public](#the-require-public-field) - Assume source files to be private by default.
    - [target-dir](#the-target-dir-field) - Default target directory.
- [[test]](#the-test-section) - The test settings.
    - [default-target](#the-default-target-field) - Set the default target for tests.
    - [auto-discovery](#the-auto-discovery-field) - Enable or disable testbench auto discovery.
- [[build]](#the-build-section) - The build settings.
    - [default-target](#the-default-target-field) - Set the default target for builds.
    - [auto-discovery](#the-auto-discovery-field) - Enable or disable top-level auto discovery.
- [[publish]](#the-publish-section) - The publish settings.
    - [default-channels](#the-default-channels-field) - Set the default channels to publish a project to.
- [[install]](#the-install-section) - The install settings.
    - [default-protocol](#the-default-protocol-field) - Set the default protocol for installing projects.
- [[vhdl-format]](#the-vhdl-format-section) - VHDL code formatting.
- [[verilog-format]](#the-verilog-format-section) - SystemVerilog/Verilog code formatting.
- [[env]](#the-env-section) - The runtime environment variables.
- [[[target]]](#the-target-array) - Define a target.
    - [name](#the-name-field) - The name of the target.
    - [description](#the-description-field) - A short description of the target.
    - [command](#the-command-field) - The command to execute the target.
    - [plans](#the-plans-field) - The list of supported blueprint file formats.
    - [build](#the-build-field) - Enable or disable target invocation for the build subcommand.
    - [test](#the-test-field) - Enable or disable target invocation for the test subcommand.
    - [[fileset]](#the-fileset-section) - Filesets to collect for the target. 
- [[[protocol]]](#the-protocol-array) - Define a protocol.
    - [name](#the-name-field) - The name of the protocol.
    - [description](#the-description-field) - A short description of the protocol.
    - [patterns](#the-patterns-field) - String patterns to match a project's URL.
    - [command](#the-command-field) - The command to execute the protocol.
- [[[channel]]](#the-channel-array) - Define a channel.
    - [name](#the-name-field) - The name of the channel.
    - [description](#the-description-field) - A short description of the channel.
    - [root](#the-root-field) - The directory where the channel exists.
    - [sync.command](#the-command-field) - The command to execute to synchronize the channel with the internet.
    - [pre.command](#the-command-field) - The command to execute immediately before the channel's publishing process.
    - [post.command](#the-command-field) - The command to execute immediately after the channel's publishing process.


### The `include` field

The `include` entry is an array of strings that are file paths to other Orbit configuration files. Relative paths are resolved relative to the directory where the configuration file that defines them exists. Only the global configuration file is allowed to have this key.

``` toml
include = [
    "profiles/p1/config.toml",
    "profiles/p2/config.toml",
    "channels/c1/config.toml"
]
```

### The `[general]` section

### The `require-public` field

The `require-public` key determines if to assume source files are private by default. When true, all source files are private to their respective project unless their file path is found in their project manifest's `public` entry. When false, all source files are treated as public when their project manifest's `public` entry is omitted.

``` toml
[general]
require-public = true
# ...
```

By default, this key is set true. This means all source files are assumed private by default, unless explicitly specified in their project manifest's `public` entry. See the project manifest's reference on the [public](./manifest.md#the-public-field) entry for more information about specifying public source files.

### The `target-dir` field

The `target-dir` key defines the default output directory to create for the planning and building phases. This value can be overridden on the command-line when the `--target-dir` option is specified. 

``` toml
[general]
# ...
target-dir = "target"
```

When this key is not defined, the default value for the build directory is "target".

### The `[test]` section

### The `default-target` field

The optional `default-target` key sets the default target when starting the build process through the test entry point (`orbit test`).

``` toml
[test]
default-target = "modelsim"
```

If the default target is set to be used and its name cannot be found among the known targets, the build process will error.

### The `auto-discovery` field

The optional `auto-discovery` key sets whether or not to allow Orbit to try to automatically discovery the testbench, if it is unambiguous.

``` toml
[test]
# ...
auto-discovery = true
```

If this field is omitted, then auto discovery is enabled by default. Auto discovery is only attempted when the user does not provide the `--dut` and `--tb` options to `orbit test`.

### The `[build]` section

### The `default-target` field

The optional `default-target` key sets the default target when starting the build process through the build entry point (`orbit build`).

``` toml
[build]
default-target = "vivado"
```

If the default target is set to be used and its name cannot be found among the known targets, the build process will error.

### The `auto-discovery` field

The optional `auto-discovery` key sets whether or not to allow Orbit to try to automatically discovery the testbench, if it is unambiguous.

``` toml
[build]
# ...
auto-discovery = true
```

If this field is omitted, then auto discovery is enabled by default. Auto discovery is only attempted when the user does not provide the `--top` option to `orbit build`.

### The `[publish]` section

### The `default-channels` field

The optional `default-channels` field is an array of strings that represent the names of known channels. When this field is provided and the current project omits the `channels` field from its manifest, the project will be published to the list of channels defined here. If the current project's manifest does have the `channels` field defined, then the `default-channels` field is not applied.

``` toml
[publish]
default-channels = ["hyperspace-labs"]
```

If the `default-channels` field is set and at least one of the names cannot be found among the known channels, the publishing process will error.

### The `[install]` section

### The `default-protocol` field

The optional `default-protocol` field can be used to specify which protocol should have priority when trying to access a project from the internet.

``` toml
[install]
default-protocol = "git"
```

If the default protocol has patterns configured and none of the patterns match the given project's repository URL, then it is not used. If the default protocol is set to be used and its name cannot be found among the known protocols, the installation process will error.

### The `[vhdl-format]` section

The currently supported entries are demonstrated in the following code snippet. Entries not present will be set to their default values.

``` toml
[vhdl-format]
# Enable colored output for VHDL code snippets
highlight-syntax = true
# Number of whitespace characters per tab/indentation
tab-size = 2
# Insert a tab before 'generic' and 'port' interface declarations
indent-interface = true
# Automatically align a signal or constant's subtype with its other identifiers
type-auto-alignment = true
# Number of whitespace characters after alignment (before the `:` character)
type-offset = 1
# Automatically align an instantiation's mapping along its port connections
mapping-auto-alignment = true
# Number of whitespace characters before port connection (before the `=>` character)
mapping-offset = 1
# Place a space between generic/port keyword and its opening parenthesis
space-interface-parenthesis = true
# The default instance name
instance-name = "ux"
```

### The `[verilog-format]` section

The currently supported entries are demonstrated in the following code snippet. Entries not present will be set to their default values. This section currently applies its settings to SystemVerilog and Verilog source code.

``` toml
[verilog-format]
# Enable colored output for code snippets
highlight-syntax = true
# Number of whitespace characters per tab/indentation
tab-size = 2
# Automatically align a port or parameter's name with the module's other names
name-auto-alignment = true
# Number of additional whitespace characters after alignment
name-offset = 0
# Number of whitespaces before a range specifier
range-offset = 1
# Automatically align an instantiation's mapping along its port connections
mapping-auto-alignment = true
# Number of whitespace characters before port connection (before the `(` character)
mapping-offset = 0
# The default instance name
instance-name = "ux"
```

### The `[env]` section

The `[env]` section allows you to set additional environment variables for target, protocol and channel command invocations. Each key's value is expected to be a string.

``` toml
[env]
LICENSE_FILE = "3000@server" 
```

By default, the variables specified will not override values that already exist in the environment. This behavior can be changed by setting the `force` flag.

Setting the `relative` flag evaluates the value as a config-relative path that is relative to the parent directory of the `.orbit` directory that contains the `config.toml` file. If the path exists, the value of the environment variable will be the full absolute path.

``` toml
[env]
TMPDIR = { value = "/home/tmp", force = true }
OPENSSL_DIR = { value = "vendor/openssl", relative = true }
```

Orbit also exposes these key/value pairs as variables for string swapping in permissible strings. See [String Swapping](./../topic/swapping.md).

Orbit protects its environment variables from being overridden, regardless if the `force` flag is set. Adding environment variables reserved by Orbit will have no effect.

### The `[[target]]` array

The `[[target]]` array lists user-defined targets for implementing the execution stage of the build process. Each target must exist under its own `[[target]]` header.

### The `name` field

The target name is an identifier used to refer to the target. It is used at the start of the build process to specify how the execution stage should behave.

``` toml
[[target]]
name = "dump-blueprint"
```

This field is required when configuring a target.

### The `description` field

The description is an optional short blurb about the target. This should be plain text (not Markdown).

``` toml
[[target]]
# ...
description = "Print the blueprint contents to the screen"
```

### The `command` field

The `command` entry for a target is a string or an array of strings ([program path with args](#executable-paths-with-arguments)) used to specify the external program and additional arguments (if they exist) to run for the execution stage of the build process.

``` toml
[[target]]
# ...
command = ["cat", "blueprint.tsv"]
```

This field is required when configuring a target.

This field supports [_string swapping_](./../topic/swapping.md).

### The `plans` field

The optional `plans` entry is an array of strings that specify which blueprint file types are supported by the target.

``` toml
[[target]]
# ...
plans = ["tsv"]
```

If more than one value is listed, then the default plan for the target is the first item in the list. 

A plan can be provided on the command-line at the start of a build process to override the default plan. If a plan is provided on the command-line, then it must first be a supported plan as well as one found within the target's defined `plans` entry.

If this field is left blank or not defined, then the default plan is "tsv".

### The `build` field

The optional `build` key explicitly allows or disallows the build entry point to the build process (`orbit build`) to use this target.

By setting this field to true, then the given target will be made available to the build entry point. By setting this field to false, then the given target is hidden and unusable for the build entry point.

``` toml
[[target]]
# ...
build = true
``` 

If this field is not defined, then the default is true.

### The `test` field

The optional `test` key explicitly allows or disallows the test entry point to the build process (`orbit test`) to use this target.

By setting this field to true, then the given target will be made available to the test entry point. By setting this field to false, then the given target is hidden and unusable for the test entry point.

``` toml
[[target]]
# ...
test = true
```

If this field is not defined, then the default is true.

### The `[fileset]` section

The `[fileset]` section allows user-defined key/value pairs, where the key corresponds to a fileset name and the value is a string that corresponds to the glob-style pattern. The fileset name will be normalized to COBOL-CASE.

If a string is the immediate value of a custom key in the `fileset` section, then it becomes the fileset's only pattern and will only be applied to the current project's directory.

``` toml
[[target]]
# ...
fileset.TEXT = "*.txt"
fileset.CPPF = "{{ orbit.tb.name }}.cpp"
```

If an inline table is the value of a custom key in the `fileset` section, then it requires the `patterns` field to be an array of strings that represent the different file patterns for this fileset. The optional `recursive` field accepts a boolean that when enabled, will apply the fileset patterns to all projects in the dependency graph.

``` toml
[[target]]
# ...
fileset.PYF = { patterns = ["*.py", "*.py3"], recursive = true }
```

The patterns for user-defined filesets support [_string swapping_](./../topic/swapping.md) (as shown in the pattern for the `CPPF` fileset).

### The `[[protocol]]` array

The `[[protocol]]` array lists user-defined protocols to access projects from the internet. Each protocol must exist under its own `[[protocol]]` header.

### The `name` field

See [[target]](#the-target-array)'s definition of [`name`](#the-name-field).

### The `description` field

See [[target]](#the-target-array)'s definition of [`description`](#the-description-field).

### The `patterns` filed

The optional `patterns` field is an array of strings that represent different URL patterns for the given protocol. If no patterns are provided, then the protocol will accept any URL from a given project. If a list of patterns is provided, then the protocol will only try a project if it's URL is a match with one or more of the patterns for that protocol.

``` toml
[[protocol]]
# ...
patterns = ["*.git"]
```

### The `command` field

The `command` entry for a protocol is a string or an array of strings ([program path with args](#executable-paths-with-arguments)) used to specify the external program and additional arguments (if they exist) to run for the download phase of the installation process.

``` toml
[[protocol]]
# ...
command = ["git", "clone", "{{ orbit.project.source }}", "-b", "{{ orbit.project.version }}"]
```

This field is required when configuring a protocol.

This field supports [_string swapping_](./../topic/swapping.md).

### The `[[channel]]` array

The `[[channel]]` array lists user-defined channels for tracking published projects. Each channel must exist under its own `[[channel]]` header.

### The `name` field

See [[target]](#the-target-array)'s definition of [`name`](#the-name-field).

### The `description` field

See [[target]](#the-target-array)'s definition of [`description`](#the-description-field).

### The `root` field

The optional `root` entry is a file system path to the root directory for the channel. Relative file paths are considered relative to the configuration file that defines this entry.

``` toml
[[channel]]
# ...
root = "./index"
```

If this entry is not defined, the default root directory for a channel is `.`, the directory where the configuration file exists.

### The `sync.command` field

The `sync.command` entry for a channel's sync hook is a string or an array of strings ([program path with args](#executable-paths-with-arguments)) used to specify the external program and additional arguments (if they exist) to run to synchronize the channel with the latest changes from the internet.

``` toml
[[channel]]
# ...
sync.command = ["git", "pull"]
```

This field is optional when configuring a channel.

This field supports [_string swapping_](./../topic/swapping.md).

### The `pre.command` field

The `pre.command` entry for a channel's pre-publish hook is a string or an array of strings ([program path with args](#executable-paths-with-arguments)) used to specify the external program and additional arguments (if they exist) to run before Orbit copies a project's metadata into the channel during the publishing process.

``` toml
[[channel]]
# ...
pre.command = ["git", "pull"]
```

This field is optional when configuring a channel.

This field supports [_string swapping_](./../topic/swapping.md).

### The `post.command` field

The `post.command` entry for a channel's post-publish hook is a string or an array of strings ([program path with args](#executable-paths-with-arguments)) used to specify the external program and additional arguments (if they exist) to run after Orbit copies a project's metadata into the channel during the publishing process.

``` toml
[[channel]]
# ...
post.command = ["git", "commit", "-am", "Publishes {{orbit.project.name}} v{{orbit.project.version}}"]
```

This field is optional when configuring a channel.

This field supports [_string swapping_](./../topic/swapping.md).
