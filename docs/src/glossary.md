# Glossary

## Available State (A)
An IP's state where the project is currently not placed on the user's machine, but
the metadata required to get it is. IP under the available state act as "pointers" to
their projects. Manifests found in vendor registry's are marked as in the Available State.

## Blueprint
A blueprint is a tab-separated file that consists of paths to all files collected 
for building an IP. It is created on `orbit plan`. Each line in blueprint consist 
of 3 values separated by tabs (fileset, identifier, filepath). If the file is an
HDL file, the identifier is the HDL library identifier. Otherwise, it is the file's name.

## Cache
The cache hosts all IP labelled under the Installed State. The cache directory is
abstracted away from the user and is not intended to be manually edited.

## Catalog
The catalog is the user's entire space of currently known IP. It consists of
the results from searching the DEV_PATH, cache, and vendor registries for manifests.

## Current Working IP
An IP in the development state that the current working directory is found. Some
commands can only be ran from a current working IP.

## Developing State (D)
An IP's state where it can be safely mutated and edited. IP in the developing state
can exist anywhere in the user's filesystem (excluding the cache folder and registry folder),
but they usually exist in the user's DEV_PATH for convenience in other functions.

## Development Path (DEV_PATH)
The path Orbit searches for IP labelled under the Developing State. IP in the DEV_PATH
are intended to be edited and developed.

## Fileset
A glob-style pattern for collecting files under a given identifier. Filesets are
used to group common files together into the blueprint during the planning phase
for future processing.

## Installed State (I)
An IP's state where it is immutable and can be safely included as a dependency into
a project under the Developing State. IP in an Installed State exist in the cache
and are not to be manually edited. If a project needs to be further developed, bring it
into the developing state and release a new version.

## Intellectual Property (IP) 
In Orbit, an IP is a project directory with a _manifest_ file at the project's 
root directory. In the context of a package manager, these are the "packages" 
Orbit manages. An IP is identified by its PKGID.

## Library
The second identifier in the PKGID. When using an IP as a dependency, the PKGID 
library identifier is also the VHDL library identifier for all primary design 
units within that IP.

## Lock File
A special file generated and maintained by Orbit outlining the IP dependencies required
reproduce the last plan. The lockfile should be checked into version control and
not manually edited by a user.

## Manifest
The Manifest is a file used to record IP-level details for Orbit. Orbit 
recognizes files named `Orbit.toml` as an IP's manifest. The manifest is 
intended to be written by the developer, although most of the details can be 
automated to Orbit.

## Package ID (PKGID)
The PKGID is a series of identifiers following __\<vendor>.\<library>.\<name>__. 
PKGIDs give IP a unique identification within Orbit and the IP catalog. No two 
IP in the catalog can have the same complete PKGID.

## Plugin
A plugin is a command invoked through the Orbit environment that executes a backend
workflow on an IP. They typically process the blueprint file and can accept additional
arguments from the command-line.

## Profile
A profile is a group of plugins, templates, and configurations under a single directory.
Profiles are useful in quickly sharing common development standards and workflows among a working
group.

## Status
The status of an IP is at which it appears in the user's catalog. There are 3
states: Developing (D), Installed (I), and/or Available (A).

## Store
A directory Orbit where maintains IP repositories for quicker installations
of subsequent IP versions.

## Template
A template is a directory available to be imported as a new IP. Templates 
support variable subsitutiton.

## Variable Subsitutiton
@todo

## Vendor
@todo

## Version
A version consists of 3 numeric values __\<major>.\<minor>.\<patch>__ to capture a
snapshot of an IP's given state. Versions are recognized in Orbit as git tags. A version
can be partially specified, having at least a __major__ value. A version can also be denoted
as `latest` or `dev`.

## VHDL
VHSIC Hardware Design Language (VHDL) is a hardware descrption language to model the
behavior of digitally electronic circuits.
