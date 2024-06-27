# Ip

Ips are the core component that Orbit operates on as a package manager. First, let's understand some key terms related to ip in the context of Orbit.

## Anatomy of an ip

A developer's tasks often involve interfacing with a collection of closely related files (source code, scripts, text files). This collection of closely related files is typically stored under a single directory and is called a _project_.

The core operations of a package manager revolve around _packages_. A _package_ is a project _with additional information provided by the developer_. This "additional information" is called _metadata_, and it is written to a special file called a _manifest_. The manifest must be placed at the project's root directory. Without a manifests, a package manager would not know which projects it should manage and what each project's current state is in relation to being a package.

In the context of being a package manager for digital hardware, Orbit calls a package an _ip_. An ip's manifest file is "Orbit.toml", with case-sensitivity.

## Working ip

Typically, developers work on one project at a given time (while we can work on projects concurrently, we unfortunately are not parallel processors...yet). The _working ip_ is the ip that is currently being developed at a given moment. The working ip is found by Orbit by checking along the working directory and its parent directories. Some Orbit commands only work when they are called within the working ip (`orbit lock`, `orbit build`).

## Types of files inside an ip

Since Orbit focuses on digital hardware projects, it automatically detects and manages files that store HDL source code. Files that store HDL source code are called _source files_. Any other files, such as scripts and test vectors, are considered _auxiliary files_.

Auxiliary files can be injected into the planning stage by specifying _filesets_ for the given target. A _fileset_ is glob-style pattern that collects matching files under a common name within the working ip. These matched files will appear in the target's generated blueprint file for future execution.

So, what files are inside an ip?
- _Source files_: Stores HDL source code (VHDL, Verilog)
- _Auxiliary files_: Any additional files that do not store source code
- _Manifest file_ (`Orbit.toml`): Stores the ip's metadata provided by the user
- _Lock file_ (`Orbit.lock`): Saves the ip's world state for reproducibility purposes
- _Ignore file_ (`.orbitignore`): Stores a list of file patterns that Orbit uses to ignore matching file paths during file discovery

All files __except the lock file__ are expected to be edited by the user. Orbit automatically maintains the lock file to ensure it can reproduce the ip's world state in the future.

### Reserved names

File names that begin with ".orbit-" are reserved for internal use and are not allowed at the root directory of an ip. Files that are named with this pattern are used by Orbit in the ip catalog to store additional metadata about the ip.

## Ip names

An ip's name is a human-readable name given to an ip so users can easily recall and locate relevant packages.

```
gates
```

An ip's _specification_, more commonly called a _spec_, is the full resolved name of an ip. As of now, the spec involves the ip's name and ip's version separated by a `:` character.

```
gates:1.0.0
```

When asking Orbit to operate on a particular ip outside of the working ip, Orbit will usually ask you to provide the ip's spec. Orbit uses the spec to lookup the ip in the catalog and then continues operation.