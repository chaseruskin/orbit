# Researching Ip

During the lifecycle of an ip, the first step in the process is typically to research existing work and identify any components which may be of value in reusing as a dependency for the new ip. This section highlights different ways to discover and gain information about existing ips and their design units.

### Assumptions

The guides in this section place no assumption on where your working directory is when running any commands; that is, these commands can be ran from any directory.

### Guides
- [Viewing existing ip](#viewing-existing-ip)
- [Gathering information about an ip](#gathering-information-about-an-ip)
- [Gathering information about a design unit](#gathering-information-about-a-design-unit)
- [Reading source code of a design unit](#reading-source-code-of-a-design-unit)

### Viewing existing ip

This guide outlines how to see what ips exist on your system in your catalog that could be potentially used as a dependency.

1. Display all results from the catalog:
```
$ orbit search
```

This will return a list of the ip, where each line records the ip name, latest known version, status in the catalog, and uuid.

### Gathering information about an ip

This guide outlines how to view more information about an ip found in your catalog that may be of particular interest.

1. Identify the name of an ip in the catalog that may be of interest:
```
$ orbit search
```

2. Return the list of all possible versions of the ip in the catalog, where `<ip>` is the name of the ip of interest:
```
$ orbit info <ip> --versions
```

3. Return the metadata found in the ip's manifest, where `<ip>` is the name of the ip of interest and `<vesion>` is the version of the ip of interest:
```
$ orbit info <ip>:<version>
```

4. Return the list of accessible design units for the ip, where `<ip>` is the name of the ip of interest and `<version>` is the version of the ip of interest:
```
$ orbit info <ip>:<version> --units
```

This will return a list of the ip's design units, where each line records the design unit's name, design unit type, and accessibility.

### Gathering information about a design unit

This guide outlines how to view a design unit's declaration for a particular design unit of an ip found in the catalog.

1. Return the design unit's declaration,  where `<unit>` is the name of the design unit, `<ip>` is the name of the ip of interest, and `<version>` is the version of the ip of interest:
```
$ orbit get <unit> --ip <ip>:<version>
```

### Reading source code of a design unit

This guide outlines how to view the source code for a particular design unit of an ip found in the catalog.

1. Display the contents from the design unit's source file, where `<unit>` is the name of the design unit, `<ip>` is the name of the ip of interest, and `<version>` is the version of the ip of interest:
```
$ orbit read <unit> --ip <ip>:<version>
```

> __Tip__: For large source files, it may be helpful to redirect the output to a temporary file, or use the `--save` flag to automatically write the results to a temporary read-only file.