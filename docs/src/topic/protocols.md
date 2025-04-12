# Protocols

A _protocol_ is a series of steps requried to get a package from the internet. Protocols exist because there are numerous ways to access data from the internet depending on your development environment. Orbit tries to be as modular and flexible as possible by introducing protocols. 

Protocols are required during the download process to acquire a package for potential cache installation.

## Default protocol

Orbit has a default protocol that relies on the Rust [`curl`](https://crates.io/crates/curl) crate to make HTTP requests. This protocol assumes the provided URLs point to a zip archive containing the targeted package. The protocol will extract the zip file to the _queue_, which is a special temporary directory handled by Orbit. Orbit generates and manages a different queue directory for each package that must be downloaded.

### Using the default protocol

To use the default protocol, modify the desired project's manifest to only specify the URL as the source. The default protocol assumes the URL points to a publicly accessible zip archive.

Filename: Orbit.toml
``` toml
[ip]
name = "orbit"
version = "1.0.0"
source = "https://github.com/chaseruskin/orbit/archive/refs/tags/1.0.0.zip"
# ...
```

## Custom protocols

A user can define a custom protocol for accessing packages from the internet by modifying the configuration file.

Orbit sets the current directory for the custom protocol execution to already be the queue directory.
This means when a custom protocol is executed, whatever files it downloads and extracts to the current directory is the directory Orbit expects to find the IP.

When a custom protocol is used, Orbit creates a temporary directory and calls the protocol's command from this newly created temporary directory. Orbit removes this temporary directory after an ip is downloaded and creates a new temporary directory for each ip download. 

### Example

One possible protocol relies on using the `git` command-line tool.

Filename: config.toml
``` toml
[[protocol]]
name = "gitit"
summary = "Access packages through git to handle remote repositories"
patterns = ["*.git"]
command = "git"
args = ["clone", "-b", "{{ orbit.ip.version }}", "{{ orbit.ip.source }}"]
```

This protocol calls `git` and clones from the IP's URL while checking out the branch/tag that matches the IP's version number. These values are resolved at runtime by Orbit through variable substitution.

More complex protocols may require using a scripting language such as Python to perform the necessary steps.

### Using a custom protocol

Edit the current ip's manifest file to specify the URL that matches one of the patterns for the configured protocol. It is each user of the package's responsibility to ensure the necessary protocol(s) are properly configured in their settings.

Filename: Orbit.toml
``` toml
[ip]
name = "orbit"
version = "1.0.0"
source = "https://github.com/chaseruskin/orbit.git"
# ...
```