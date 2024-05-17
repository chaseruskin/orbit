# Configuration

The `config.toml` file stores settings and extends `orbit`'s functionality. It is written in the [TOML](https://toml.io/en/) format. It is maintained by the developer and can be shared across teams for consistent development environments.

> __Note:__ The configuration's file name is "config.toml", with respect to case-sensitivity.

## Paths

When a field is expected to be a filesystem path, `orbit` has the ability to resolve relative paths. The path is determined in relation to the currently processed `config.toml`'s parent directory. This design choice was implemented in order to allow for path definitions to be valid across developer machines when sharing configurations. It is recommended to use relative paths when setting a path to a field in a `config.toml`.

## Precedence

Orbit supports multiple levels of configuration. Each level has its own order of precedence:

1. Local configuration file (location: current working IP)

2. Global configuration file (location: `$ORBIT_HOME`)

3. Configuration files listed in the global `config.toml`'s [`include`](#the-include-field) (items in the array are processed in order; first-to-last)

The configuration files are processed in the order defined above. When a configuration file defines a field, no other configuration files later in the process will be able to override its value. If a field is never provided an explicit value, the hard-coded defaults will be used.

> __Tip:__ You can modify some values in the configuration file through the command-line by using the `orbit config` command.

Every configuration file consists of the following sections:

- [include](#the-include-field) - Lists other `config.toml` files to process.
- [[general]](#the-general-section) - The general settings.
    - [build-dir](#the-build-dir-field) - Default build directory.
    - [language-mode](#the-language-mode-field) - HDL language(s) to enable.
- [[vhdl-format]](#the-vhdl-format-section) - VHDL code formatting.
- [[env]](#the-env-section) - The runtime environment variables.
- [[[plugin]]](#the-plugin-array) - Define a plugin.
    - [name](#the-name-field) - The name of the plugin.
    - [description](#the-description-field) - A short description of the plugin.
    - [command](#the-command-field) - The command to execute the plugin.
    - [args](#the-args-field) - Arguments to pass to the command.
    - [[fileset]](#the-fileset-section) - Filesets to collect for the plugin.
    - [explanation](#the-explanation-field) - A detailed description of the plugin. 
- [[[protocol]]](#the-protocol-array) - Define a protocol.
    - [name](#the-name-field) - The name of the protocol.
    - [description](#the-description-field) - A short description of the protocol.
    - [command](#the-command-field) - The command to execute the protocol.
    - [args](#the-args-field) - Arguments to pass to the command.
    - [explanation](#the-explanation-field) - A detailed description of the protocol.

### The `include` field

``` toml
include = [
    "profiles/p1/config.toml",
    "profiles/p2/config.toml"
]
```

### The `[general]` section

### The `build-dir` field

Define the default output directory to create for the planning and building phases. This value can be overridden on the command-line when the `--build-dir` option is available. When this field is not defined, the default value for the build directory is "build".

``` toml
[general]
build-dir = "build"
# ...
```

### The `language-mode` field

Enable specific HDLs to be read by `orbit`. Supports the following options: "vhdl", "verilog", or "mixed". When this field is not defined, the default value for the language mode is "mixed" (enabling all supported HDLs).

``` toml
[general]
language-mode = "mixed"
```

### The `[vhdl-format]` section

The currently supported entries are demonstrated in the following code snippet. Entries not present will be set to their hard-coded default value.

``` toml
[vhdl-format]
# enable colored output for generated vhdl code
highlight-syntax = true
# number of whitespace characters per tab/indentation
tab-size = 2
# insert a tab before 'generic' and 'port' interface declarations
indent-interface = true
# automatically align a signal or constant's subtype with its other identifiers
type-auto-alignment = false
# number of whitespace characters after alignment (before ':' token)
type-offset = 0
# automatically align a instantiation's mapping with its other identifiers
mapping-auto-alignment = false
# number of whitespace characters after mapping (before '=>' token)
mapping-offset = 1
```

### The `[env]` section

The user can define an arbitrary number of their own entries with their determined value represented in string format.

``` toml
[env]
# accessible as ORBIT_ENV_FOO
foo = "0"
# accessible as ORBIT_ENV_SUPER_BAR
super-bar = "1"
```

### The `[[plugin]]` array

### The `name` field

### The `description` field

### The `command` field

### The `args` field

### The `explanation` field

### The `[fileset]` section

### The `[[protocol]]` array

### The `name` field

See [[plugin]](#the-plugin-array)'s definition.

### The `description` field

See [[plugin]](#the-plugin-array)'s definition.

### The `command` field

See [[plugin]](#the-plugin-array)'s definition.

### The `args` field

See [[plugin]](#the-plugin-array)'s definition.

<!--
## config.toml

The first config file you may come across is `config.toml`. This file is used to load initial startup settings into orbit and customize a user's program experience.

Here is a very minimal and basic example config file:
``` toml
include = ["profiles/ks-tech/config.toml"]

[env]
QUARTUS_PATH = "C:/IntelFPGA_lite/19.1/quartus/bin64"

[[plugin]]
alias = "zipr"
description = "Compress files into a submission-like format"
command = "python"
args = ["./main/plugins/zipr.py"]
fileset.zip-list = "submission.txt"

[[protocol]]
name = "zip-op"
description = "Handle zip file urls"
command = "python"
args = ["./main/protocols/download.py"]
```

The __home configuration__ is the config.toml file located at your $ORBIT_HOME path.

If you have `cat` installed, you can view your home config file in the console:
```
$ cat "$(orbit env ORBIT_HOME)/config.toml"
```

> __Tip:__ You can modify some values in the configuration file through the command-line by using the `orbit config` command.

## Paths

When specifying a value that is known to be a path, Orbit supports resolving relative paths in relation to the config.toml's path it is currently reading. This allows for a path value to be correct out-of-the-box across users and machines when sharing configurations.

## Precedence

Orbit supports multiple levels of configuration. The order of precedence:

1. local configuration file (located in current IP)

2. global configuration file (located in $ORBIT_HOME)

3. configuration files listed in `include` entry (last has higher precedence than first)

A key's value is overridden upon a configuration file of higher precedence also setting a previously defined key from a lower-precedence file.

## Entries

The following is a list of acceptable entries (key/value pairs) recognized by Orbit in configuration files (`config.toml`).


### `include` : _list_ of _string_
- paths to other configurations files to load before the home configuration
- only supported in the home configuration file

``` toml
include = ["profiles/ks-tech/config.toml"]
```

### `[env]` : _table_
- user-defined additional keys to set as runtime environment variables during build phase
- the following example would set an environment variable ORBIT_ENV_VAR_1 as "100" during runtime

``` toml
[env]
VAR_1 = "100"
# ...
```

<!-- 
### `core.build-dir` : _string_
- directory to create to save blueprint file to
- default is "build"

``` toml
[core]
build-dir = "target"
# ...
```

### `core.user` : _string_
- your name
- useful for template variable substitution

``` toml
[core]
user = "Kepler [KST-001]"
# ...
```

### `core.date-fmt` : _string_
- date formatting for template variable substitution
- default is `"%Y-%m-%d"`
- see chrono's [documentation](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html#specifiers) for complete list of formatting specifiers

``` toml
[core]
date-fmt = "%B %e, %Y" # July 8, 2001
# ...
``` 

### `[[plugin]]` : _array of tables_
- `alias` : _string_ 
    - plugin name to reference when invoking
    - required
- `command` : _string_
    - first argument to pass to subprocess
    - required
- `description` : _string_
    - short description about the plugin
- `args` : _array_ of _string_
    - additional arguments to follow command in subprocess  
- `fileset` : _inline table_
    - user-defined additional keys to store glob-style file patterns
- `explanation` : _string_
    - long description about the plugin

``` toml
[[plugin]]
alias   = "vvd"
command = "vivado"
description = "Basic toolflow for Vivado Design Suite"
args    = ["-mode", "batch", "-source", "script.tcl"]
fileset.EDA-FLOW    = "*.tcl"
fileset.CONSTRAINTS = "*.xdc"
explanation = """\
    This plugin runs Vivado in non-project mode to perform its tasks.

Usage:
    orbit build --plugin vvd -- [options]

Options:
    -tclarg mode=<num>      0 - synth, 1 - impl, 2 - bit

Environment:
    ORBIT_ENV_VIVADO_PATH   Local path to Vivado binaries   

Dependencies:
    Vivado Design Suite (tested: 2019.2)
"""
```

### `[[protocol]]` : _array of tables_
- `name` : _string_ 
    - protocol name to reference in an IP's manifest
    - required
- `command` : _string_
    - first argument to pass to subprocess
    - required
- `description` : _string_
    - short description about the protocol
    - optional
- `args` : _array_ of _string_
    - additional arguments to follow command in subprocess 
    - optional 
- `explanation` : _string_
    - long description about the protocol
    - optional

``` toml
[[protocol]]
name = "git-op"
description = "Fetch remote repositories using git"
command = "git"
args = ["clone", "-b", "{{ orbit.ip.version }}", "{{ orbit.ip.source.url }}", "{{ orbit.queue }}/{{ orbit.ip.name }}"]
explanation = """\
This protocol tries to clone a repository defined under the source URL at a tag 
matching the IP's version.

Examples:
    [ip]
    # ...
    name = "lab1"
    version = "1.0.0"
    source = { protocol = "git-op", url = "https://github.com/path/to/lab1.git" }
    # ...

Dependencies:
    git (tested: 2.36.0)
"""
```
-->