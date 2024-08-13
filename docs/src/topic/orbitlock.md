# Orbit.lock

The _Orbit.lock_ lock file is a file that captures every dependency required for the current ip. This includes information about exact versions of dependencies and how to get them if any are missing from the cache.

The lock file is managed by Orbit, and formalizes the data the user provided in the `Orbit.toml` manifest file. The lock file is required for every ip and should not be manually edited.

With a lock file, the current state of the ip can be reproduced at a later time and in any environment. If the current ip uses version control, then it is recommended to track `Orbit.lock` to ensure reproducibility across environments.

To update the current ip's lock file, use `orbit lock`. The lock file will also automatically be updated before the build process when using `orbit build` or `orbit test`.

> __Note:__ An ip's lock file contains all the data required by it to reproduce its current state, so it does not require reading the lock files of its dependencies.