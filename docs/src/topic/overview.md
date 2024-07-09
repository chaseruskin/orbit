# Overview

Orbit is an agile package manager and extensible build tool for HDLs.

## Key concepts

- Every ip requires a manifest file (`Orbit.toml`). This is maintained by the developer. The manifest file documents basic metadata and the project's list of direct dependencies.

- Backend EDA tools and workflows (makefiles, TCL scripts, etc.) are decoupled from ip and are able to be reused across projects by creating targets in the configuration file (`config.toml`).

- Orbit does not require a version control system (VCS). Orbit is intended to work with any VCS (git, mercurial, svn, etc.).

- Orbit solves the namespace collision problem by a variant of name mangling when primary design unit identifiers conflict in the dependency tree (_dynamic symbol transformation_).

- Downloading an ip stores a compressed snapshot of the ip to install later. Downloads are placed your ip catalog's _archive_, which is a special directory to Orbit that is abstracted away from the user.

- Installing an ip places the source code in a place for it to be referenced in other projects. Installations are located in your ip catalog's _cache_, which is a special directory to Orbit and is abstracted away from the user. Files are not allowed to be modified in the cache.

- Orbit generates a lockfile (`Orbit.lock`) during the planning stage of the build process. The lockfile saves the entire state such that Orbit can return to this state at a later time or on a different computing system. All necessary data that is required to reproduce the build is stored in the lockfile. The lockfile is maintained by Orbit and should be checked into versionc control.

- The build process is occurs in two stages: planning and execution. These stages happen sequentially and together when calling `orbit test` or `orbit build`.

- Orbit generates a blueprint during the planning stage of the build process. The blueprint is a single file that lists the HDL source code files required for the particular build in topologically sorted order. Targets can also specify other file types to be collected into the blueprint. The blueprint is an artifact to be consumed by the target's process during the exection stage of the build process. Since it can frequently change with each build, it should not be checked into version control.
