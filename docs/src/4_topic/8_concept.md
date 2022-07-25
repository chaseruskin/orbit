# Conceptual Overview

Orbit is a frontend package manager for HDL development leaving the door open for flexible backend tooling and workflows.

## Key points

- Every IP requires a manifest file (`Orbit.toml`). This is maintained by the developer. The manifest file documents basic metadata and the project's list of direct dependencies.

- Backend tools and workflows (makefiles, TCL scripts, etc.) are able to be decoupled from IP and can be reused across projects by setting up plugins.

- Orbit avoids dependency hell by a form of name mangling when primary design unit identifiers conflict in the dependency tree (_dynamic symbol transformation_).

- Place an IP on your DEV_PATH to make edits and new development. The DEV_PATH should be a known path on your machine to place projects currently being developed.

- Install an IP to the cache to reuse it in another project (`orbit install`). The cache is a hidden directory abstracted away from the user and is maintained through commands to Orbit.

- Orbit generates a lock file (`Orbit.lock`) during the planning phase (`orbit plan`) after resolving the dependency tree to store all the data required to reproduce the build. The lock file is maintained by Orbit and should be checked into version control.

- Orbit generates a blueprint file (`blueprint.tsv`) during the planning phase which lists the in-order HDL files required to build the design The blueprint may also list other user-defined filesets. The blueprint file is maintained by Orbit. It changes frequently and is placed in the build directory, so it should not be checked into version control.

- In general, plugins will read the blueprint file to analyze the source files and then perform some action using a backend tool.

- It is required to plan a design (`orbit plan`) before building a design (`orbit build`).

- Launching a new version performs a series of checks to make sure the version will work with Orbit when being referenced in other projects. A version is detected by a semver git tag.