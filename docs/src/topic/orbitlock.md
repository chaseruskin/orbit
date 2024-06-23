# Orbit.lock

`Orbit.lock` is a special file automatically created and updated by Orbit. It is not intended to be manually edited. Orbit writes this file every time it needs to prepare for a target's execution (typically during `orbit plan` or `orbit run`). The purpose of the lock file is store the information necessary to reproduce the state in how Orbit prepared the target for execution. Ideally, if a project has a lock file, then the project can be prepared again in exactly the same way (and therefore rebuilt in the same way) on any machine to reproduce identical results from that particular target on all machines.

Orbit uses the lock file when it determines the current ip's `Orbit.toml` manifest data matches with the lock file entry written for the current ip. When this comparison is true it signals that there has been no change to the state of the system. Any change to `Orbit.toml` may result in an updated `Orbit.lock` file.

It is recommended to check in the lock file to version control to ensure the project can be rebuilt on other machines when the repository is cloned.

> __Note:__ An ip will only read its own lock file and not the lock file of any of its dependencies when needing data to reproduce a build.