# __orbit lock__

## __NAME__

lock - save the world state of an ip

## __SYNOPSIS__

```
orbit lock [options]
```

## __DESCRIPTION__

This command saves the state of the world according to the working ip. To
accomplish this, Orbit reads working ip's manifest file to resolve any
missing dependencies. It's writes all the information that is necessary to
reproduce this state to the ip's lock file, Orbit.lock.

This command can only be ran within a working ip.

It is encouraged to check the lock file into version control such that others
trying to build the ip can reproduce the ip's current state. The lock file
should not be manually edited by the user.

To capture the state of the world according to the ip, this command will
download and install any unresolved dependencies. If an installed dependency's 
computed checksum does not match the checksum stored in the lock file, it 
assumes the installation to be corrupt and will re-install the dependency to 
the cache.

## __OPTIONS__

`--force`  
      Ignore reading the precomputed lock file

## __EXAMPLES__

```
orbit lock
orbit lock --force
```

