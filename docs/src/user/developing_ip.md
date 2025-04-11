# Developing Ip

Once existing work has been evaluated and the requirements for an ip are defined, the next step is typically to begin developing the actual logic for the ip. This section walks through the common steps to building new ip and integrating with existing ip.

### Assumptions

The guides in this section, except for the ones on creation and initialization of an ip, assume you are running commands from the root directory or any subdirectory of the current ip. The current ip, also sometimes referred to as the local ip, is the ip being actively developed.

This section also assumes you have one or more targets already configured. To learn how to create targets, see [Creating Targets](./creating_targets.md).


### Guides
- [Creating a new ip](#creating-a-new-ip)
- [Initializing an existing project as an ip](#initializing-an-existing-project-as-an-ip)
- [Integrating a design unit internal to the current ip](#integrating-a-design-unit-internal-to-the-current-ip)
- [Integrating a design unit external to the current ip](#integrating-a-design-unit-external-to-the-current-ip)
- [Viewing the design hierarchy](#viewing-the-design-hierarchy)
- [Viewing available targets for testing](#viewing-available-targets-for-testing)
- [Running a test](#running-a-test)
- [Viewing available targets for building](#viewing-available-targets-for-building)
- [Running a build](#running-a-build)

## Creating a new ip

1. Create a new directory as an Orbit ip by also creating a basic manifest file, where `<ip>` is the name of the new directory as well as the name of the newly created ip:
```
$ orbit new <ip>
```

2. Enter the root directory for the newly created ip to begin working within it, where `<ip>` is the name of the newly created ip's directory:
```
$ cd <ip>
```

## Initializing an existing project as an ip

1. Enter the root directory for the existing project, where `<project>` is the root directory:
```
$ cd <project>
```

2. Initialize the current working directory as an Orbit ip by creating a basic manifest file, where `<ip>` is the name of the newly initialized ip:
```
$ orbit init --name <ip>
```

## Integrating a design unit internal to the current ip

1. Add the necessary code in the current ip's source files referencing the internal design unit as normal. For design units that are entities or modules, return code snippets that can be used to declare IO signals and instantiate the design unit:
```
$ orbit get <unit> --signals --instance
```

2. Verify the dependencies are correctly discovered in the design hierarchy:
```
$ orbit tree --edges all --format long
```

## Integrating a design unit external to the current ip

1. Add the external ip as a dependency to the current ip's manifest, where `<ip>` is the name of the external ip and `<version>` is the version of the external ip:
``` toml
[dependencies]
# ...
<ip> = "<version>"
```

2. Add the necessary code in the current ip's source files referencing the external design unit as normal. For design units that are entities or modules, return code snippets that can be used to declare IO signals and instantiate the design unit:
```
$ orbit get <unit> --ip <ip>:<version> --signals --instance
```

3. Verify the dependencies are correctly discovered in the design hierarchy:
```
$ orbit tree --edges all --format long
```

For more ways on how to add dependencies, see [Specifying Dependencies](./specifying_deps.md).

## Viewing the design hierarchy

1. Return the list of known design units for the current ip:
```
$ orbit info --units
```

2. View the design hierarchy for a particular design unit within the current ip, where `<unit>` is the name of the design unit of interest:
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
