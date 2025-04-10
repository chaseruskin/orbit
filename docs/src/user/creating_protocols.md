# Creating Protocols

A protocol is a user-defined series of steps requried to obtain an ip from the internet. This section provides steps for ways to create and configure a protocol.

> __Note__: The default method for obtaining ip from the internet is using Rust's `curl` library under the assumption that the ip's URL is a .zip file. If you prefer a different method to access ip from the internet, then you must create a protocol.

### Assumptions

If a step mentions the "protocol configuration", this corresponds to the entry for the protocol that has been defined in an Orbit configuration file (config.toml).

### Guides

- [Configuring a protocol](#configuring-a-protocol)
- [Viewing available protocols](#viewing-available-protocols)
- [Using a protocol](#using-a-protocol)

## Configuring a protocol

This guide walks through how to add a protocol to be recognized by Orbit. In this guide, our protocol is called `gitit`.

1. Open an Orbit configuration file.
   
2. Make a new entry in the `[[protocol]]` array:
``` toml
[[protocol]]
name = "gitit"
description = "Download ip using git"
command = "git"
args = ["clone", "{{ orbit.ip.source.url }}"]
```

A protocol may be as simple as a list of known command-line arguments, or may require invoking a script written in a scripting language such as Python or Tcl.

> __Tip__: The strings within the array for the `args` field of a protocol support string variables. See [String Swapping](./../topic/swapping.md#protocol-arguments) to view what variables are allowed.


## Viewing available protocols

This guide walks through how to view the protocols already configured and available to use for an ip.

1. Return the list of configured protocols:
``` 
$ orbit install --list
```

## Using a protocol

This guide walks through how to configure a particular ip to use a previously configured protocol called `gitit`.

1. Open the current ip's manifest file.

2. Add the `source` field to the ip's manifest while making sure to specify the `url` and `protocol`, where the `url` key contains the ip's git repository and the `protocol` key contains the name of previously configured protocol we wish to use (`gitit` in this example):
``` toml
[ip]
# ...
source = { url = "https://github.com/chaseruskin/gates.git", protocol = "gitit" }
```