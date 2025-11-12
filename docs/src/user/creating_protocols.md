# Creating Protocols

A protocol is a user-defined series of steps requried to obtain a project from the internet. This section provides steps for ways to create and configure a protocol.

> __Note__: The default method for obtaining a project from the internet is using Rust's `curl` library under the assumption that the project's URL is a .zip file. If you prefer a different method to access projects from the internet, then you must create a protocol.

### Assumptions

If a step mentions the "protocol configuration", this corresponds to the entry for the protocol that has been defined in an Orbit configuration file (config.toml).

### Guides

- [Configuring a protocol](#configuring-a-protocol)
- [Configuring a protocol as default](#configuring-a-protocol-as-default)
- [Viewing available protocols](#viewing-available-protocols)
- [Using a protocol](#using-a-protocol)

## Configuring a protocol

This guide walks through how to add a protocol to be recognized by Orbit. In this guide, our protocol is called `gitit`.

1. Open an Orbit configuration file.
   
2. Make a new entry in the `[[protocol]]` array:
``` toml
[[protocol]]
name = "gitit"
description = "Download projects using git"
command = ["git", "clone", "{{orbit.project.repository}}", "-b", "{{orbit.project.version}}"]
patterns = ["*.git"]
```

A protocol may be as simple as a list of known command-line arguments, or may require invoking a script written in a scripting language such as Python or Tcl.

> __Tip__: The strings within the array for the `args` field of a protocol support string variables. See [String Swapping](./../topic/swapping.md#protocol-arguments) to view what variables are allowed.

## Configuring a protocol as default

This guide walks through how to configure an existing protocol to be the first to check if it can be used for a given project's repository URL.

1. Open an Orbit configuration file.

2. Add the `default-protocol` field in the `[install]` section, where the value is the name of a previously defined protocol (such as `gitit`):
``` toml
[install]
default-protocol = "gitit"
```


## Viewing available protocols

This guide walks through how to view the protocols already configured and available to use for a project.

1. Return the list of configured protocols:
``` 
$ orbit install --list
```

2. Return configuration data about a particular protocol, such as `gitit`:
```
$ orbit install --list --protocol gitit
```

## Using a protocol

This guide walks through how to configure a particular project to use a previously configured protocol called `gitit`.

1. Open the current project's manifest file.

2. Add the `repository` field to the project's manifest, which contains a string of the project's git repository:
``` toml
[project]
# ...
repository = "https://github.com/chaseruskin/gates.git"
```

Since this repository ends with `.git`, it matches the pattern for the previously configured `gitit` protocol, so Orbit will use the `gitit` command and arguments to download the project.