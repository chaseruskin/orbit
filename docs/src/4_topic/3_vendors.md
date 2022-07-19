# Vendors and Registries

It can be hard to begin to keep track of all your IP that is available, especially among a team where different members are contributing to different projects. To streamline maintaining a list of the IP available, Orbit uses _vendors_.

The vendor is the first identifier in an IP's PKGID. A _registry_ is a vendor that is an actual directory holding Orbit IP manifest files.

Registries are special types of directory that can indirectly point to a collection of IP. They "point" to IP by storing the manifest files corresponding to each version of the IP.

Once a vendor has a directory initialized and set up, Orbit automatically handles the ability to refresh, use, and update the registry.

## index.toml

A vendor is recognized by Orbit with a `index.toml` file at the root of the vendor's directory.

``` toml
[vendor]
name       = "ks-tech"
summary    = "in-house IP for space communications"
repository = "<repository-url>" # optional
```

## Tracking IP

In order for an IP to be tracked by a vendor, the IP's `vendor` field in the `Orbit.toml` file must match the `name` field in the `vendor.toml` file. The IP must also have a remote repository url stored in the `repository` field that can be shared and used to clone the repository to be tracked by an IP. A vendor itself does not require a remote repository url.

_index.toml_
```toml
[vendor]
name = "ks-tech"
repository = "<remote-repository-for-ks-tech>"
```

_Orbit-0.2.3.toml_
``` toml
[ip]
name    = "gates"
library = "rary"
vendor  = "ks-tech"     # matches `name` for the vendor from `vendor.toml`
version = "0.2.3"
repository = "<remote-repository-url-for-gates>"
```

The convention is to place vendors in a `vendor` folder at your ORBIT_HOME location. 

Orbit finds the available IP from within the root of vendor directories by matching all files with `Orbit-*.toml` file names.

## Hooks

Orbit automates registry management. However, Orbit also gives you the flexibility in how to upload new releases with each registry.

_index.toml_
``` toml
# ...
[hook]
pre-publish = "./pre-publish.hook"
post-publish = "./post-publish.hook"
```

Hooks are a series of commands ran during a particular point in one of Orbit's underlying processes. In this case, the pre-publish hook is called __before__ Orbit places the new manifest copy into the registry. The post-publish hook is called __after__ Orbit places the new manifest copy into the registry.

> __Note:__ Variable substitution is supported in hook files.

### Examples 

The following pre-publish hook file ensures the project does not have any unsaved changes before it refreshes the repository and checks out to a new branch for the specific launch.

example: _pre-publish.hook_
```
git restore .
git remote update
git checkout -b {{ orbit.ip }}-{{ orbit.ip.version }}
```

The following post-publish hook file commits the newly created manifest copy and pushes it to a new remote branch before returning to its original branch.

example: _post-publish.hook_
```
git add {{ orbit.ip.library }}/{{orbit.ip.name }}/Orbit-{{ orbit.ip.version }}.toml
git commit -m "Adds {{ orbit.ip }} {{ orbit.ip.version }}"
git push --set-upstream origin {{ orbit.ip }}-{{ orbit.ip.version }}
git checkout -
```