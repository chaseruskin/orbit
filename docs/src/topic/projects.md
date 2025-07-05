# Projects

A _project_ is a collection of one or more IP cores. Projects are the _packages_ Orbit manages as a package manager. First, let's understand some key terms related to packages in the context of Orbit.

## Terminology

A developer's tasks often involve interfacing with a collection of closely related files (source code, scripts, text files). This collection of closely related files is typically grouped together under a single directory called a project. Orbit recognizes a directory on your filesystem as a project by finding its _manifest_, which contains metadata about the project, such as its name and version. The manifest file must be named "Orbit.toml" and exists at the root of the project.

In the context of being a package manager for digital hardware, Orbit calls its packages _projects_ to disambiguite from package constructs found within HDLs. For more information, see [Packages and Cores](./packages_and_cores.md).

## Working project

Typically, developers work on one project at a given time (while we can work on projects concurrently, we unfortunately are not parallel processors). The _working project_ is the project that is currently being developed at a given moment. Orbit identifies the working project by checking along the working directory and its parent directories for a manifest file. Once a manifest file is found, Orbit considers this the working project. Some Orbit commands only work when they are called within the working project (`orbit lock`, `orbit build`).

The working project may also be called other names, such as the _local project_ or _current project_. These name differences are only differences in name as the same meaning is preserved.

## Anatomy of a project

Since Orbit focuses on digital hardware projects, it automatically detects and manages files that store HDL source code. Files that store HDL source code are called _source files_. Any other files, such as scripts and test vectors, are considered _auxiliary files_.

Auxiliary files can be injected into the planning stage by specifying custom _filesets_ for the given target. A _fileset_ is glob-style pattern that collects matching files under a common name within the working project. These matched files will appear in the target's generated blueprint file for future execution.

So, what files are inside a project?
- _Source files_: Stores HDL source code (VHDL, Verilog, SystemVerilog)
- _Auxiliary files_: Any additional files that do not store source code
- _Manifest file_ (`Orbit.toml`): Stores the project's metadata provided by the user
- _Lock file_ (`Orbit.lock`): Saves the project's world state for reproducibility purposes

All files __except the lock file__ are expected to be edited by the user. Orbit automatically maintains the lock file to ensure it can reproduce the project's world state in the future.

### Reserved names

File names that begin with ".orbit-" are reserved for internal use and are not allowed at the root directory of a project. Files that are named with this pattern are used by Orbit in the catalog to store additional metadata about the project.

## Names

A project's name is a human-readable identifier given to an project so users can easily remember and locate packages of interest.

```
gates
```

A project's _specification_, more commonly called a _spec_, is the full resolved name of an project. The spec involves the project's name, uuid, and version. A complete spec looks like the following:

```
gates+8ah2qa261k8wgv55sd1qq17w9:1.0.0
```

When asking Orbit to operate on a particular project outside of the working project, Orbit will ask you to provide the project's spec. Orbit uses the spec to lookup the project in the catalog and then carry out the requested function on that project. 

To learn more about an project spec, see [Project ID Specification](./../reference/project_id_specification.md).