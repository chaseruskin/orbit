# Configuration

The configuration data Orbit processes is stored in TOML files. TOML files have a file extension `.toml` and store key-value pairs. To learn more about TOML files, visit their [website](https://toml.io/en/).

## config.toml

The first config file you may come across is `config.toml`. This file is used to load initial startup settings into orbit and customize a user's application experience.

Here is a very minimal and basic example config file:
``` toml
include = [
    "profile/kepler-space-tech/config.toml"
]

[core]
path = "c:/users/kepler/hdl" # path to find and store IP in-development
editor = "c:/users/kepler/appdata/local/programs/vscode/code"
user = "kepler"

```

## Orbit.toml

This is the manifest file for all valid Orbit IP packages. 

Here is a very minimal and basic example manifest file:

``` toml
[ip]
name    = "gates"
library = "rary"
version = "0.1.0"
vendor  = "ks-tech"
summary = "low-level combinational gate logic"

[dependencies]

```

To learn more about the manifest, see the manifest page.