# Configuration

The configuration data Orbit processes is stored in TOML files. TOML files have a file extension `.toml` and store key-value pairs. To learn more about TOML files, visit their [website](https://toml.io/en/).

> __Note:__ Throughout this page, an __entry__ refers to a key-value pair.

## config.toml

The first config file you may come across is `config.toml`. This file is used to load initial startup settings into orbit and customize a user's program experience.

Here is a very minimal and basic example config file:
``` toml
include = ["profile/ks-tech/config.toml"]

[core]
path = "c:/users/kepler/hdl" # path to find and store IP in-development
editor = "c:/users/kepler/appdata/local/programs/vscode/code"
```

The __home configuration__ is the config.toml file located at your ORBIT_HOME path.

If you have `cat` installed, you can view your home config file in the console:
```
$ cat "$(orbit env ORBIT_HOME)/config.toml"
```

> __Tip:__ You can modify some values in the configuration file through the command-line by using the `orbit config` command.

## Paths

When specifying a value that is known to be a path, Orbit supports resolving relative paths in relation to the config.toml's path it is currently reading. This allows for a path value to be correct out-of-the-box across users and machines when sharing configurations.

## Precedence

Orbit supports multiple levels of configuration. The order of precedence:

1. local configuration file (located in current ip)

2. global configuration file (located in ORBIT_HOME)

3. configuration files listed in `include` entry (last has higher precedence than first)

A key's value is overridden upon a configuration file of higher precedence also setting a previously defined key from a lower-precedence file.

## Entries

The following is a list of acceptable entries (key/value pairs) recognized by Orbit in configuration files (`config.toml`).


### `include` : _list_ of _string_
- paths to other configurations files to load before the home configuration
- only supported in the home configuration file

``` toml
include = ["profile/ks-tech/config.toml"]
```

### `core.path` : _string_
- development path
- contains mutable ip (in-development)

``` toml
[core]
path = "C:/users/chase/projects/"
# ...
```

### `core.editor` : _string_
- program called to open files/folders

``` toml
[core]
editor = "code"
# ...
```

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
- `summary` : _string_
    - short description about the plugin
- `args` : _array_ of _string_
    - additional arguments to follow command in subprocess  
- `fileset` : _inline table_
    - user-defined additional keys to store glob-style file patterns
- `details` : _string_
    - long description about the plugin

``` toml
[[plugin]]
alias   = "main"
command = "vivado"
summary = "basic toolflow for vivado"
args    = ["-mode", "batch", "-source", "script.tcl"]
fileset.FLOW   = "*.tcl"
fileset.PINOUT = "*.xdc"
details = """\
Usage:
    orbit build --plugin vivado -- [options]

Options:
    -tclarg mode=<num>      0 - synthesis, 1 - implementation, 2 - bitstream

Description:
    This plugin runs vivado in non-project mode to perform its tasks.
"""
```

### `[env]` : _table_
- user-defined additional keys to set as runtime environment variables during build phase
- the following example would set an environment variable ORBIT_ENV_VAR_1 as "100" during runtime

``` toml
[env]
VAR_1 = "100"
# ...
```

### `[[template]]` : _array of tables_
- `alias` : _string_
    - template name to reference when invoking
    - required
- `path` : _string_
    - root directory path to copy
    - required
- `ignore` : _list_ of _string_
    - glob-style patterns to ignore during creating a new project

``` toml
[[template]]
alias  = "base"
path   = "template/"
ignore = ["extra/"]
```

### `vendor.index` : _array of strings_
- paths to vendor index files to load vendors
- if the path is relative, it is relative to the `config.toml` file that defines it

``` toml
[vendor]
index = [
    'profile/ks-tech/vendor/index.toml'
]
```