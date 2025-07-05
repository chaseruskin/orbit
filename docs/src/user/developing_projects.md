# Developing Projects

Once existing work has been evaluated and the requirements for a project are loosely defined, the next step is typically to begin developing the actual logic for the project. This section walks through the common steps to build a new project and integrate it with existing projects.

### Assumptions

The guides in this section, except for the ones on creation and initialization of a project, assume you are running commands from the root directory or any subdirectory of the current project. The current project is the local project being actively developed.

This section also assumes you have one or more targets already configured. To learn how to create targets, see [Creating Targets](./creating_targets.md).

### Guides
- [Creating a new project](#creating-a-new-project)
- [Initializing an existing directory as a project](#initializing-an-existing-directory-as-a-project)
- [Integrating a design unit internal to the current project](#integrating-a-design-unit-internal-to-the-current-project)
- [Integrating a design unit external to the current project](#integrating-a-design-unit-external-to-the-current-project)
- [Viewing the design hierarchy](#viewing-the-design-hierarchy)
- [Viewing available targets for testing](#viewing-available-targets-for-testing)
- [Running a test](#running-a-test)
- [Viewing available targets for building](#viewing-available-targets-for-building)
- [Running a build](#running-a-build)

## Creating a new project

1. Create a new directory as an Orbit package by also creating a basic manifest file, where `<project>` is the name of the new directory as well as the name of the newly created project:
```
$ orbit new <project>
```

1. Enter the root directory for the newly created project to begin working within it, where `<project>` is the name of the newly created project's directory:
```
$ cd <project>
```

## Initializing an existing directory as a project

1. Enter the root directory for the existing project, where `<project>` is the root directory:
```
$ cd <project>
```

2. Initialize the current working directory as an Orbit package by creating a basic manifest file, where `<project>` is the name of the newly initialized project:
```
$ orbit init --name <project>
```

## Integrating a design unit internal to the current project

1. Add the necessary code in the current project's source files referencing the internal design unit as normal. For design units that are entities or modules, return code snippets that can be used to declare IO signals and instantiate the design unit:
```
$ orbit get <unit> --signals --instance
```

2. Verify the dependencies are correctly discovered in the design hierarchy:
```
$ orbit tree --edges all --format long
```

## Integrating a design unit external to the current project

1. Add the external project as a dependency to the current project's manifest, where `<project>` is the name of the external project and `<version>` is the version of the external project:
``` toml
[dependencies]
# ...
<project> = "<version>"
```

2. Add the necessary code in the current project's source files referencing the external design unit as normal. For design units that are entities or modules, return code snippets that can be used to declare IO signals and instantiate the design unit:
```
$ orbit get <unit> --project <project>:<version> --signals --instance
```

3. Verify the dependencies are correctly discovered in the design hierarchy:
```
$ orbit tree --edges all --format long
```

For more ways on how to add dependencies, see [Specifying Dependencies](./specifying_deps.md).

## Viewing the design hierarchy

1. Return the list of known design units for the current project:
```
$ orbit info --units
```

2. View the design hierarchy for a particular design unit within the current project, where `<unit>` is the name of the design unit of interest:
```
$ orbit tree <unit>
```

## Viewing available targets for testing

1. View the list of available targets configured to run using the test entry point:
```
$ orbit test --list
```

2. Return the configuration data of a particular test target, where `<target>` is the name of the target of interest:
```
$ orbit test --list --target <target>
```

## Running a test

1. Start the build process using the test entry point for a particular testbench and particular target, where `<unit>` is the testbench name of interest and `<target>` is the name of the desired target:
```
$ orbit test --tb <unit> --target <target>
```

2. Inspect the resulting command-line outputs and any files generated to the target's output directory.

> __Tip__: You can also provide additional arguments on the command-line to the target's command that will run during the execution stage of the build process. Add any arguments after an empty flag `--` for Orbit to skip processing them and instead pass them to the target's command.
> ```
> $ orbit test --tb <unit> --target <target> -- <args>...
> ```


## Viewing available targets for building

1. View the list of available targets configured to run using the build entry point:
```
$ orbit build --list
```

2. Return the configuration data of a particular build target, where `<target>` is the name of the target of interest:
```
$ orbit build --list --target <target>
```

## Running a build

1. Start the build process using the build entry point for a particular top-level design unit and particular target, where `<unit>` is the top-level name of interest and `<target>` is the name of the desired target:
```
$ orbit build --top <unit> --target <target>
```

2. Inspect the resulting command-line outputs and any files generated to the target's output directory.

> __Tip__: You can also provide additional arguments on the command-line to the target's command that will run during the execution stage of the build process. Add any arguments after an empty flag `--` for Orbit to skip processing them and instead pass them to the target's command.
> ```
> $ orbit build --top <unit> --target <target> -- <args>...
> ```
