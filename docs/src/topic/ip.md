# Ip

Ips are the core component that Orbit operates on as a package manager. First, let's understand some key terms related to ip in the context of Orbit.

## Terminology

A developer's tasks often involve interfacing with a collection of closely related files (source code, scripts, text files). This collection of closely related files is typically stored under a single directory and is called a _project_.

The core operations of a package manager revolve around _packages_. A _package_ is a project _with additional information provided by the developer_. This "additional information" is called _metadata_, and it is written to a special file called a _manifest_. The manifest must be placed at the project's root directory. Without manifests, a package manager would not know which projects it should manage and what each project's current state is in regards to being a package.

In the context of being a package manager for digital hardware, Orbit calls a package an _ip_. An ip's manifest file is "Orbit.toml", with case-sensitivity.

## Working ip

Typically, developers work on one project at a given time (while we can work on projects concurrently, we unfortunately are not parallel processors). The _working ip_ is the ip that is currently being developed at a given moment. Orbit identifies the working ip by checking along the working directory and its parent directories for a manifest file. Once a manifest file is found, Orbit considers this the working ip. Some Orbit commands only work when they are called within the working ip (`orbit lock`, `orbit build`).

The working ip may also be called other names, such as the _local ip_ or _current ip_. These name differences are only differences in name as the same meaning is preserved.

## Anatomy

Since Orbit focuses on digital hardware projects, it automatically detects and manages files that store HDL source code. Files that store HDL source code are called _source files_. Any other files, such as scripts and test vectors, are considered _auxiliary files_.

Auxiliary files can be injected into the planning stage by specifying _filesets_ for the given target. A _fileset_ is glob-style pattern that collects matching files under a common name within the working ip. These matched files will appear in the target's generated blueprint file for future execution.

So, what files are inside an ip?
- _Source files_: Stores HDL source code (VHDL, Verilog)
- _Auxiliary files_: Any additional files that do not store source code
- _Manifest file_ (`Orbit.toml`): Stores the ip's metadata provided by the user
- _Lock file_ (`Orbit.lock`): Saves the ip's world state for reproducibility purposes

All files __except the lock file__ are expected to be edited by the user. Orbit automatically maintains the lock file to ensure it can reproduce the ip's world state in the future.

### Reserved names

File names that begin with ".orbit-" are reserved for internal use and are not allowed at the root directory of an ip. Files that are named with this pattern are used by Orbit in the ip catalog to store additional metadata about the ip.

## Names

An ip's name is a human-readable identifier given to an ip so users can easily remember and locate packages of interest.

```
gates
```

An ip's _specification_, more commonly called a _spec_, is the full resolved name of an ip. The spec involves the ip's name, uuid, and version. A complete spec looks like the following:

```
gates+8ah2qa261k8wgv55sd1qq17w9:1.0.0
```

When asking Orbit to operate on a particular ip outside of the working ip, Orbit will ask you to provide the ip's spec. Orbit uses the spec to lookup the ip in the catalog and then carry out the requested function on that ip. 

To learn more about an ip spec, see [Ip Specification](./../reference/names.md).