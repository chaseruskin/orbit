# Vendors and Registries

It can be hard to begin to keep track of all your IP that is available, especially among a team where different members are contributing to different projects. To streamline maintaining a list of the IP available, Orbit uses _vendors_.

The vendor is the first identifier in an IP's PKGID. To store a 

A vendor is special type of directory that can indirectly point to a collection of IP. A vendor "points" to IP by storing the manifest files corresponding to each version of the IP.

Once a vendor directory is initialized and set up, Orbit automatically handles the ability to refresh, use, and update the directory.

## vendor.toml

A vendor is recognized by Orbit with a `vendor.toml` file at the root of the vendor's directory.

``` toml
[vendor]
name       = "ks-tech"
summary    = "in-house IP for space communications"
repository = "<repository-url>" # optional

# ...
```

The `vendor.toml` file is also responsible for keeping track of all the IP it contains. The index is updated by Orbit when a new IP is launched for the first time with `orbit launch`.

``` toml
# ...

[index]
rary.gates   = "<remote-repository-url>" 
rary.math    = "<remote-repository-url>" 
util.toolbox = "<remote-repository-url>" 
```

## Tracking IP

In order for an IP to be tracked by a vendor, the IP's `vendor` field in the `Orbit.toml` file must match the `name` field in the `vendor.toml` file. The IP must also have a remote repository url stored in the `repository` field that can be shared and used to clone the repository to be tracked by an IP. A vendor itself does not require a remote repository url.

vendor.toml
```toml
[vendor]
name = "ks-tech"

[index]
rary.gates = "<remote-repository-url>"
```

Orbit.toml
``` toml
[ip]
library = "rary"
name    = "gates"
vendor  = "ks-tech"     # matches `name` for the vendor from `vendor.toml`
version = "0.2.3"
repository = "<remote-repository-url>"
```

The convention is to place vendors in a `vendor` folder at your ORBIT_HOME location. Orbit will automatically check if a folder called `vendor` exists and will search it for any `vendor.toml` files.