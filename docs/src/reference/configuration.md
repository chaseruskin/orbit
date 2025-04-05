# Configuration

The `config.toml` file stores settings and extends Orbit's functionality. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and can be shared across teams for consistent development environments.

> __Note:__ The configuration's file name is "config.toml", with respect to case-sensitivity.

## Paths

When a field is expected to be a file system path, Orbit has the ability to resolve relative paths. The path is determined in relation to the currently processed `config.toml`'s parent directory. 

This design choice was implemented to allow path definitions to be valid across file systems when sharing configurations. It is recommended to use relative paths when setting a field's value as a path in a `config.toml`.

## Hierarchical structure

Orbit supports multiple levels of configuration. Each level has its own order of precedence. The order of precedence is the following:

1. Local configuration file (current ip's `.orbit/conifg.toml`)

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
- [[build]](#the-build-section) - The build settings.
    - [default-target](#the-default-target-field) - Set the default target for builds.
- [[vhdl-format]](#the-vhdl-format-section) - VHDL code formatting.
- [[verilog-format]](#the-verilog-format-section) - SystemVerilog/Verilog code formatting.
- [[env]](#the-env-section) - The runtime environment variables.
- [[[target]]](#the-target-array) - Define a target.
    - [name](#the-name-field) - The name of the target.
    - [description](#the-description-field) - A short description of the target.
    - [command](#the-command-field) - The command to execute the target.
    - [args](#the-args-field) - Arguments to pass to the command.
    - [plans](#the-plans-field) - The list of supported blueprint file formats.
    - [build](#the-build-field) - Enable or disable target invocation for the build subcommand.
    - [test](#the-test-field) - Enable or disable target invocation for the test subcommand.
    - [[fileset]](#the-fileset-section) - Filesets to collect for the target. 
- [[[protocol]]](#the-protocol-array) - Define a protocol.
    - [name](#the-name-field) - The name of the protocol.
    - [description](#the-description-field) - A short description of the protocol.
    - [command](#the-command-field) - The command to execute the protocol.
    - [args](#the-args-field) - Arguments to pass to the command.
- [[[channel]]](#the-channel-array) - Define a channel.
    - [name](#the-name-field) - The name of the channel.
    - [description](#the-description-field) - A short description of the channel.
    - [root](#the-root-field) - The directory where the channel exists.
    - [sync.command](#the-command-field) - The command to execute when synchronizing the channel.
    - [sync.args](#the-args-field) - Arguments to pass to the command during synchronization.
    - [pre.command](#the-command-field) - The command to execute immediately before launch.
    - [pre.args](#the-command-field) - Arguments to pass to the command immediately before launch.
    - [post.command](#the-command-field) - The command to execute immediately after launch.
    - [post.args](#the-args-field) - Arguments to pass to the command immediately after launch.


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

The `require-public` key determines if to assume source files are private by default. When true, all source files are private to the ip their created within unless their file path is found in ip manifest's `public` entry. When false, all source files are treated as public when the ip manifest's `public` entry is omitted.

``` toml
[general]
require-public = true
# ...
```

By default, when this key is not specified, it is set true. See the ip manifest's reference on the [public](./manifest.md#the-public-field) entry for more information about defining public source files.

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

The `default-target` key sets the default target when starting the build process through the test entry point (`orbit test`).

``` toml
[test]
default-target = "modelsim"
```

If the default target is set to be used and it cannot be found among the known targets, it will error.

### The `[build]` section

### The `default-target` field

The `default-target` key sets the default target when starting the build process through the build entry point (`orbit build`).

``` toml
[build]
default-target = "vivado"
```

If the default target is set to be used and it cannot be found among the known targets, it will error.

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

The `[env]` section allows for an arbitrary number of user-defined key/value pairs.
Each key's value is expected to be a string. Orbit exposes these key/value pairs as variables for string swapping in permissible strings as well as environment variables during the execution stage of the build process.

``` toml
[env]
# Accessible as environment variable: ORBIT_ENV_FOO, string variable: orbit.env.foo
FOO = "bar" 
# Accessible as environment variable: ORBIT_ENV_LICENSE_FILE, string variable: orbit.env.license.file
LICENSE_FILE = "3000@server" 
```

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

The `command` entry for a target is used to specify what program to run for the execution stage of the build process.

``` toml
[[target]]
# ...
command = "cat"
```

This field is required when configuring a target.

### The `args` field

The optional `args` entry is an array of strings that are passed to the program when it runs during the execution stage of the build process. Relative file paths included in this value are considered relative to the configuration file that defines this entry.

``` toml
[[target]]
# ...
args = ["blueprint.tsv"]
```

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

The `[fileset]` section allows user-defined key/value pairs, where the key corresponds to a fileset name and the value is a string that corresponds to the glob-style pattern.

``` toml
[[target]]
# ...
fileset.text = "*.txt"
fileset.pymdl = "{{ orbit.tb.name }}.py"
```

The values for user-defined filesets support [_string swapping_](./../topic/swapping.md) (as shown in second example entry).

### The `[[protocol]]` array

The `[[protocol]]` array lists user-defined protocols to access ips from the internet. Each protocol must exist under its own `[[protocol]]` header.

### The `name` field

See [[target]](#the-target-array)'s definition of [`name`](#the-name-field).

### The `description` field

See [[target]](#the-target-array)'s definition of [`description`](#the-description-field).

### The `command` field

See [[target]](#the-target-array)'s definition of [`command`](#the-command-field).

### The `args` field

See [[target]](#the-target-array)'s definition of [`args`](#the-args-field).

This field supports [_string swapping_](./../topic/swapping.md).

### The `[[channel]]` array

The `[[channel]]` array lists user-defined channels for tracking published ips. Each channel must exist under its own `[[channel]]` header.

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

See [[target]](#the-target-array)'s definition of [`command`](#the-command-field).

### The `sync.args` field

See [[target]](#the-target-array)'s definition of [`args`](#the-args-field).

This field supports [_string swapping_](./../topic/swapping.md)

### The `pre.command` field

See [[target]](#the-target-array)'s definition of [`command`](#the-command-field).

### The `pre.args` field

See [[target]](#the-target-array)'s definition of [`args`](#the-args-field).

This field supports [_string swapping_](./../topic/swapping.md)

### The `post.command` field

See [[target]](#the-target-array)'s definition of [`command`](#the-command-field).

### The `post.args` field

See [[target]](#the-target-array)'s definition of [`args`](#the-args-field).

This field supports [_string swapping_](./../topic/swapping.md)
