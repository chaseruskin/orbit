# Protocols

A _protocol_ is a series of steps requried to get a package from the internet. Protocols exist because there are numerous ways to access data from the internet depending on your development environment. Orbit tries to be as modular and flexible as possible by introducing protocols. 

Protocols are required during the download process to acquire a package for potential cache installation.

The goal of a protocol is to place a project's contents on the user's local file system. Every protocol's process is started from a temporary directory handled by Orbit. After a protocol runs its process, Orbit will try to find the requested project's manifest within this temporary directory. If Orbit finds the project's manifest, then the download is considered successful. Orbit creates and manages a different temporary directory for each project being downloaded.

## Standard protocol

Orbit has one pre-defined standard protocol that relies on the Rust [`curl`](https://crates.io/crates/curl) crate to make HTTP requests. This protocol assumes the provided URLs point to a zip archive containing the targeted package. The standard protocol will extract the zip file to the temporary directory handled by Orbit. If the project's source file is not a zip file, then this protocol will not work. 

If your requirements for how you point a project's source contents will not work with the standard protocol, then consider configuring a custom protocol.

## Custom protocols

Users can configure their own processes to run when trying to download a project from the internet by adding a new `[[protocol]]` entry in an Orbit configuration file.

Orbit will start the custom protocol's process in a temporary directory for the custom protocol to try to download the project's contents from its provided source URL. Orbit removes the temporary directory after a project is successfully or unsuccessfully downloaded and creates a new temporary directory for each project download.

## Examples

One possible protocol relies on using the `git` command-line tool.

Filename: config.toml
``` toml
[[protocol]]
name = "gitit"
summary = "Access packages through git to handle remote repositories"
command = ["git", "clone", "-b", "{{orbit.project.version}}", "{{orbit.project.source}}"]
patterns = ["*.git"]
```

This protocol calls `git` and clones from the project's URL while checking out the branch/tag that matches the project's version number. These values are resolved at runtime by Orbit through variable substitution.

This protocol only picks up source URLs that match the `*.git` glob pattern. If a custom protocol does not define a `patterns` field for its configured entry, then the protocol will be pick up all source URLs. 

> __Note__: If a custom protocol defines a empty list of patterns in the `patterns` field for its configured entry, then the protoocl will _never_ pick up any source URLs.
> ``` toml
> [[protocol]]
> # ...
>patterns = []
> ```

More complex protocols may require using a scripting language such as Python to perform the necessary steps.
