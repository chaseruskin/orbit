# Protocols

A protocol is a series of steps to take to get a package from the internet. Protocols exist because there are numerous ways to access data from the internet depending on your development team and environment. Orbit tries to be as modular and flexible as possible with protocols. They are required during the download step in acquiring a package for future reference within the cache.

## Default Protocol

Orbit has a default protocol that relies on the Rust [`curl`](https://crates.io/crates/curl) crate to make HTTP requests. This protocol assumes the URLs inputted into it point to a zip archive containing the targeted package. The protocol will extract the zip file, and then place the contents in the _queue_, which is a special directory marked by Orbit as its "waiting room" for packages to be later installed to the cache.

## Custom Protocols

A user can define a custom protocol for accessing packages from the internet by modifying `config.toml`.

_config.toml_
``` toml
[[protocol]]
name = "git-op"
summary = "Accesses packages through git to handle remote repositories."
command = "git"
args = ["clone", "-b", "{{ orbit.ip.version }}", "{{ orbit.ip.source.url }}", "{{ orbit.queue }}/{{ orbit.ip.name }}"]
```

The example above demonstrates one way to use the `git` command-line tool to clone packages from the internet through remote repositories. More complex protocols may require using a scripting language such as Python to program the necessary steps.

### How to use the default protocol for a package

Modify the current project's `Orbit.toml` file to only specify the URL as the source. The default protocol assumes the URL points to a publicly accessible zip archive.

_Orbit.toml_
``` toml
[ip]
name = "orbit"
version = "0.9.3"
source = "https://github.com/c-rus/orbit/archive/refs/tags/0.9.3.zip"
# ...
```

### How to set a custom protocol for a package

Modify the current project's `Orbit.toml` file to specify the URL as well as the protocol required. It is each user of the package's responsibility to ensure the necessary protocol(s) are properly configured in their settings.

_Orbit.toml_
``` toml
[ip]
name = "orbit"
version = "0.9.3"
source = { url = "https://github.com/c-rus/orbit.git", protocol = "git-op" }
# ...
```


