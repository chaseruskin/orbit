# Packages

Packages are a group of related files that fall within the same scope. Packages are more commonly called _intellectual property_, or IP, in the hardware design space.

## Package Identifier

The package identifier is a project's unique string of characters following a specification. It is a single name and is defined under the "name" field in the IP's manifest. Every IP is required to have a name.

### Rules

The following rules currently apply to a package identifier:

- begins with an ASCII letter (`a-z`, `A-Z`)
- contains only ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), dashes `-`, and underscores `_`

## Package Specification

A package specification, or _spec_, is the total unambiguous reference to a particular IP at a particular version.

```
spec ::= <name>[:version]
```

### Specification Grammar

```
pkgid ::= [[<vendor>.]<library>.]<name>
```

### Example Specifications
The following provides various valid inputs when defining an IP's spec value and how it decomposes into its parts.

Spec          | Name | Version |         
--------------|------|---------|            
gates:1.0.0   |gates |1.0.0    |    
ram           |ram   |latest   |
fifo:2.3      |fifo  |2.2.X    |  

### Namespace Collisions

Within a user's _catalog_, two different specs may share common identifiers. Two identifiers are considered equivalent if their lowercase mapping is identical, where dashes (`-`) also map to underscores (`_`).

Spec 1        | Spec 2  | Collision |         
--------------|---------|-----------|            
gates         |GATES    |true       |    
ram           |rom      |false      |
fifo_cdc      |Fifo-CDC |true       |  

> __Note:__ A resolution to this problem is to add an IP's UUID to the package specification. While each IP already has a UUID auto-assigned in their lock file, this is a proposed feature that has yet to be implemented.

## Package Library

A package can optionally belong to a _library_. A library is defined under the "library" field in the IP's manifest and follows the same rules as the package identifier. When producing a blueprint file, it will choose to associate all HDL files within a particular package with the provided library. If no library is defined in the package's manifest, then the default `work` library is provided. 

