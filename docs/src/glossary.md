# Glossary

### Project
A project is a collection of HDL source files and any other required files related to a specific application or library. Placing a manifest in a project makes it an IP.

### Package
See [IP](#intellectual-property-ip).

### Intellectual Property (IP) 
An IP is a project with a manifest file at its root directory. At a minimum, an IP has two attributes: a name and a version.

### Manifest
A manifest is a file that decscribes an IP recognized by `orbit`. Manifest files 
are exactly named `Orbit.toml`. The manifest is intended to be written by the user.

### Current Working IP (CWIP)
The current working IP (CWIP) is the IP detected from the current working directory on the command-line. Some commands can only be executed from the CWIP, such as `plan` and `build`.

## Catalog
The catalog is the user's entire space of currently known IP to `orbit`. It consists of 3 main layers: cache, downloads, channels.

### Cache
The cache is the location where immutable references to a specific IP's version exist. Dependencies to an IP are referenced from the cache. The IP's at the cache level are considered _installed_.

### Downloads
The downloads is the location where compressed snapshots of a specific IP's version exist. The compressed IP files are unable to be referenced as dependencies, but they are able to be _installed_ to the cache for usage. The IP's at the downloads level are considered _downloaded_.

### Channels
The channels are a set of decentralized registries that store the manifests for versions of IP. No source code is stored in a channel, however, `orbit` is able to use the manifest as means to _download_ an IP to the downloads for local filesystem access. The IP's at the channels level are considered _available_. Users are encouraged to create and share their own channels.

### Blueprint
A blueprint is a tab-separated file that lists all the necessary files needed to perform a particular build for the CWIP. HDL files are listed in topologically-sorted order from top to bottom, while other files can be included through user-defined filesets. 

Each line in a blueprint consists of 3 values separated by tabs (fileset, identifier, filepath). If the file is an HDL file, the identifier is the HDL library identifier, otherwise, it is the file's name.

### Fileset
A fileset is a glob-style pattern for collecting files under a given name. Filesets are
used to group common files together into the blueprint during the planning phase
for future processing by a plugin during the building phase.

### Lockfile
A lockfile is a file that exactly describes an IP's dependencies. It is generated and maintained by `orbit`. The lockfile should be checked into your version control system for reproducible builds. It is not to be manually edited by the user. 

From the lockfile, `orbit` is able to download missing dependencies, install missing dependencies, and verify the data integrity of installed dependencies.

### IP Specification (spec)
The spec describes the format for identifying and referencing an IP. Each IP in the user's catalog must have a unique spec. The complete spec is: `<name>[:<version>]`.

### Plugin
A plugin is a user-defined command able to be called during the building phase. A plugin typically follows 3 steps: 
1. Parse the blueprint
2. Process the referenced files
3. Generate an output product

Plugins can accept additional arguments from the command-line and define additional filesets to be collected during the planning phase. Users are encouraged to create and share their own plugins.

### Profile
A profile is a user-defined group of plugins, settings, and/or channels under a single directory. A profile does not necessarily have to have all listed aspects in order to be considered a "profile".

Profiles are useful for quickly sharing and maintaining common development standards and workflows among a team environment.

### VHDL
VHSIC Hardware Design Language (VHDL) is a hardware descrption language to model the behavior of digitally electronic circuits.

### Orbit.toml
See [manifest](#manifest).

### Orbit.lock
See [lockfile](#lockfile).
