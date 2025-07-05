# Researching Projects

During the lifecycle of a project, the first step in the process is typically to research existing work and identify any components which may be of value in reusing as a dependency for the new project. This section highlights different ways to discover and gain information about existing projects and their design units.

### Assumptions

The guides in this section place no assumption on where your working directory is when running any commands; that is, these commands can be ran from any directory.

### Guides
- [Viewing existing projects](#viewing-existing-project)
- [Gathering information about a project](#gathering-information-about-a-project)
- [Gathering information about a core](#gathering-information-about-a-core)
- [Reading source code of a core](#reading-source-code-of-a-core)

## Viewing existing projects

This guide outlines how to see what projects exist on your system in your catalog that could be potentially used as a dependency.

1. Display all results from the catalog:
```
$ orbit search
```

This will return a list of the known projects, where each line records the project's name, latest known version, status in the catalog, and UUID.

## Gathering information about a project

This guide outlines how to view more information about a project found in your catalog that may be of particular interest.

1. Identify the name of a project in the catalog that may be of interest:
```
$ orbit search
```

1. Return the list of all possible versions of the project in the catalog, where `<project>` is the name of the project of interest:
```
$ orbit info <project> --versions
```

1. Return the metadata found in the project's manifest, where `<project>` is the name of the project of interest and `<vesion>` is the version of the project of interest:
```
$ orbit info <project>:<version>
```

1. Return the list of accessible IP cores for the project, where `<project>` is the name of the project of interest and `<version>` is the version of the project of interest:
```
$ orbit info <project>:<version> --units
```

This will return a list of the project's design units, where each line records the design unit's name, design unit type, and accessibility.

## Gathering information about a design unit

This guide outlines how to view a design unit's declaration for a particular design unit of a project found in the catalog.

1. Return the design unit's declaration,  where `<unit>` is the name of the design unit, `<project>` is the name of the project of interest, and `<version>` is the version of the project of interest:
```
$ orbit get <unit> --project <project>:<version>
```

## Reading source code of a design unit

This guide outlines how to view the source code for a particular design unit of a project found in the catalog.

1. Display the contents from the design unit's source file, where `<unit>` is the name of the design unit, `<project>` is the name of the project of interest, and `<version>` is the version of the project of interest:
```
$ orbit read <unit> --project <project>:<version>
```

> __Tip__: For large source files, it may be helpful to redirect the output to a temporary file, or use the `--save` flag to automatically write the results to a temporary read-only file.