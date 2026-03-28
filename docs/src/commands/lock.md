# __orbit lock__

## __NAME__

lock - save the world state of a project

## __SYNOPSIS__

```
orbit lock [options]
```

## __DESCRIPTION__

Saves the state of the world for the local project. To accomplish this, Orbit 
reads the local project's manifest file, "Orbit.toml", to resolve any missing
project dependencies. It writes the information required to reproduce this 
state to the project's lock file, "Orbit.lock".

A local project must exist for this command to execute.

It is encouraged to check the lock file into version control such that other
users trying to reconstruct the project can reproduce the project's current 
state. The lock file should not be manually edited by the user.

To capture the world state for the local project, Orbit downloads and installs 
any unresolved project dependencies. If an installed dependency's computed 
checksum does not match the checksum stored in the lock file, it assumes the 
installation is corrupt and will reinstall the dependency to the cache.

## __OPTIONS__

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Ignore reading the precomputed lock file

## __EXAMPLES__

```
orbit lock
orbit lock --force
```

