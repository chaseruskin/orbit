# Ip Names

In order to identify an ip among others, Orbit requires developers to assign a human-readable name to each created ip.

The ip name is a unique string of characters that abides by a certain set of rules. It is a single name and is defined under the "name" field in an ip's manifest. Every ip is required to have a name.

## Rules

The following rules currently apply to a ip name:

- begins with an ASCII letter (`a-z`, `A-Z`)
- contains only ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), dashes `-`, and underscores `_`

## Ip specification

An ip specification, commonly abbreviated to _spec_, is the total unambiguous reference to a particular ip at a particular version.

```
spec ::= <name>[:version]
```

### Example specifications

The following provides various valid inputs when defining an ip's spec and how it decomposes into its parts.

Spec          | Name | Version |         
--------------|------|---------|            
gates:1.0.0   |gates |1.0.0    |    
ram           |ram   |latest   |
fifo:2.3      |fifo  |2.2.X    |  

### Namespace Collisions

Two different ip's may share a common name within the catalog even though their contents are different. Two names are considered equal if their lowercase mapping is identical, where dashes (`-`) also map to underscores (`_`).

Spec 1        | Spec 2  | Collision |         
--------------|---------|-----------|            
gates         |GATES    |true       |    
ram           |rom      |false      |
fifo_cdc      |Fifo-CDC |true       |  

> __Note:__ A resolution to this problem is to add an ip's UUID to the ip specification. While each ip already has a UUID auto-assigned in their lock file, this is a proposed feature that has yet to be implemented.

## Ip libraries

An ip can optionally belong to a library. An ip's _library_ is a higher-level scope that loosely groups together multiple ips. This library identification is used for grouping the HDL source code itself into their language-defined libraries as well.

A library is defined through the "library" field in the ip's manifest file. It's format follows the same rules as the ip's name. If no library is defined in the ip's manifest, the the default "work" library is assigned to the files of the project.

When referencing files through library definitions in the working ip, the library will always be "work", regardless of the value for the "library" field. The "library" field's value goes into effect when the design units of that ip are referenced as a depndency in separate ip.

