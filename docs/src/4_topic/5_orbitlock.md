# Orbit.lock

`Orbit.lock` is a special file automatically created and updated by Orbit. It is not intended to be manually edited. Orbit writes this file every time `orbit plan` is called. The purpose of the lock file is store the necessary information to reproduce the build. Ideally, if a project has a lock file, then the project can be rebuilt on any machine and reproduce identical results on all machines.

Orbit uses the lock file when it determines the current ip's `Orbit.toml` manifest data matches with the lock file entry written for the current ip. When this comparison is true it signals that there has been no change to the state of the system. Any change to `Orbit.toml` may result in an updated `Orbit.lock` file.

It is recommended to check in the lock file to version control to ensure the project can be rebuilt on other machines when the repository is cloned.

> __Note:__ An IP will only read its own lock file and not the lock file of any of its dependencies when needing data to reproduce a build.