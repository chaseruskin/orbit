# IP Core Visibility

Scope, accessibility, and visibility are important concepts when writing code intended to be used by a greater audience. Specifying what is strictly internal to an package (_private_) and what is accessible outside of an package (_public_) can help with encapsulation, code organization, and managing complexity. Specifying accessibility for the modules within an package can keep the package's contents focused and express designer's intent on which modules should be used by the outside world.

In traditional hardware description languages such as VHDL and SystemVerilog, there already exists this concept of scope and accessibility: modules external to one another can only access the other through its ports, leaving the internal architectures private to the modules that define them. Some signals are private to a module, that is, they are defined and used strictly internal to that module. A user of that module does not need to know the details of why that signal exists, rather, they simply need to understand what the module does at a system-level and how to communicate with it through its interface.

However, at the design unit abstraction level, where users deal with the primary building blocks of these languages such as `entity`, `module`, and `package`, there is no sense of accessibility. All design units are accessible because this is the highest level of abstraction defined for these languages; there is nothing more abstract to encapsulate them. But with Orbit, two layers of abstraction are introduced: [_packages_ and _cores_](./packages_and_cores.md). An package encapsulates one or more cores, which encapsulate one or more design units, thus providing the opportunity to introduce scope and accessibility at the design unit abstraction level. Users can specify which source files, which in turn store design units, are public or private with respective to the package that encapsulates them.

## Approach to accessibility

A good rule of thumb when thinking about accessibility is to make everything as private as possible; make accessible only what is strictly necessary. In the context of packages, this typically means making the following private to an package:

- __Testbenches__: These modules are typically only used when actively developing a project, not when it is needed as a dependency. A source file that contains a testbench is typically named with a `_tb` suffix.
- __Board-level design units__: These modules are typically designed around application-specific requirements, stitching together the physical board's interface with the core logic of the application. An application would require its own board-level design unit, thus not making it a good candidate for accessibility within a project. A source file that contains a board-level design unit is typically named with `top` or `app`.

Any other design units that should be left private are up to the developer and greatly depend on how a project's code is organized.

## File accessibility

By default, all source files, and therefore all design units, are considered private to the current project.

A project's manifest allows for users to set the [`public`](./../reference/manifest.md#the-public-field) field, which can store a list of user-defined file patterns for Orbit to consider publicly accessible from outside the project.

## File visibility

By default, all source files in a project are included when Orbit performs source code analysis within the current project.

A project's manifest allows for users to set an [`ignore`](./../reference/manifest.md#the-ignore-field) field, which can store a list of user-defined file patterns for Orbit to ignore during file discovery.

### Resolving errors

Orbit prevents duplicate primary design units from being identified within certain situations. For example, duplicate design unit names are not allowed within the same project because Orbit cannot resolve ambiguity in which unit is used where.

An error may look like the following:
```
error: duplicate primary design units identified as "foo"

location 1: rtl/foo1.vhd:20:1
location 2: rtl/foo2.vhd:1:1

hint: resolve this error by either
    1) renaming one of the units to a unique identifier
    2) adding one of the file paths to the manifest's "project.ignore" field
```

The `ignore` field can be used in this scenario to tell Orbit to ignore reading a particular file during the HDL source code dependency analysis.

Filename: Orbit.toml
``` toml
[project]
# ...
ignore = [
    "rtl/foo2.vhd"
]
```

In this example, the value for the above `ignore` field in the local project's manifest will resolve the previous error because it prevents Orbit from seeing the file "rtl/foo2.vhd" during any file discovery operations.

## File pattern format

Both the `ignore` key and the `public` key take an array of strings as their value. These strings are interpreted as file patterns, which follow the [.gitignore](https://git-scm.com/docs/gitignore#_pattern_format) format.
