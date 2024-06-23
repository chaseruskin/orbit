<h1 align="center"><code>orbit</code></h1>

<div align="center">
  <a href="https://github.com/cdotrus/orbit/actions">
    <img src="https://github.com/cdotrus/orbit/workflows/pipeline/badge.svg" alt="pipeline">
  </a>
  <a href="https://cdotrus.github.io/orbit/">
    <img src="https://github.com/cdotrus/orbit/actions/workflows/docs.yml/badge.svg" alt="docs">
  </a>
  <a href="https://www.gnu.org/licenses/gpl-3.0">
    <img src="https://img.shields.io/badge/License-GPLv3-blue.svg" alt="License: GPL v3">
  </a>
  <a href="https://hub.docker.com/repository/docker/crus800/orbit/general">
    <img src="https://img.shields.io/badge/dockerhub-images-important.svg?logo=docker" alt="images">
  </a>
  <a href="https://github.com/cdotrus/orbit/releases">
    <img src="https://img.shields.io/github/downloads/cdotrus/orbit/total.svg" alt="downloads">
  </a>
</div>
<br>

`orbit` is a package manager for Hardware Description Languages (HDL). 

Read the [Book of Orbit](https://cdotrus.github.io/orbit/) for complete documentation.

<br>

`orbit` manages your projects, called IPs, by handling the overhead for referencing, maintaining, and integrating your hardware description files:

```
cpu/
├─ Orbit.toml
├─ rtl/
│  ├─ ctrl.vhd
│  ├─ datapath.vhd
│  └─ top.vhd
└─ sim/
   └─ top_tb.vhd
```

<br>

Adding a simple TOML file `Orbit.toml` to a directory denotes that directory as an IP to `orbit`:

``` toml
[ip]
name = "cpu"
version = "1.0.0"

[dependencies]
gates = "2.0.0"
```

<br>

`orbit` generates VHDL code snippets able to be directly inserted into a new VHDL design for rapid reuse:
```
$ orbit get and_gate --ip gates:2.0.0 --library --signals --instance
```
``` vhdl
library gates;

signal a : std_logic;
signal b : std_logic;
signal x : std_logic;

u_and_gate : entity gates.and_gate
  port map(
    a => a,
    b => b,
    x => x
  );
```

<br>

`orbit` plans your build by generating a file list, called a blueprint, that contains a list of the required files for your given design in topologically-sorted order to act as an input to any backend toolchain:

```
VHDL-RTL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/nand_gate.vhd
VHDL-RTL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/and_gate.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/datapath.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/ctrl.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/top.vhd
VHDL-SIM	work	/users/chase/projects/cpu/sim/top_tb.vhd
```

## Features

`orbit` has a lot more useful features relating to HDL package management and development:

- `orbit` is a frontend package manager, not a build system, so it allows users to define and automate their own workflows for building HDL designs.

- Linux, MacOS, and Windows are supported with no additional dependencies.

- Docker images of `orbit` are available for easy integration into new or existing CI/CD pipelines.

- A GitHub Action is available to install orbit for GitHub workflows using [`cdotrus/setup-orbit`](https://github.com/cdotrus/setup-orbit.git).

- Reproducible builds are achieved with checksums and automatic handling of a lockfile `Orbit.lock`. 

- Namespace collisions, a problem inherent to VHDL and not resolved in many backend tools, is solved through a custom algorithm called [_dynamic symbol transformation_](https://cdotrus.github.io/orbit/topic/dst.html).

- Multiple versions of the same entity (or more broadly, entities given the same identifier) are allowed in the same build under [two simple constraints](https://cdotrus.github.io/orbit/topic/dst.html#limitations).

- Navigate HDL source code efficiently to read its inline documentation and visit its implementation through `orbit`'s ability to locate HDL code segments

- Produce VHDL code snippets with a single command to properly instantiate entities within a new design.

- Quickly search through your IP catalog by filtering based on keywords, catalog status, and name.

- Avoid being locked into a specific vendor's tooling through `orbit`'s common interface with a flexible build command to adapt to any workflow.
  
- `orbit` is version control system (VCS) agnostic through defining custom protocols for fetching IP. Continue to use your current VCS (or none).

- Minimal user upkeep is required to maintain a manifest file `Orbit.toml` that identifies an IP, its metadata, and any dependencies from your catalog.

- View the current design's tree hierarchy at an HDL entity level or at an IP level.

- Implement custom scripted workflows through a plugin system.

- Specify additional supportive files to be passed to your backend workflows with filesets.

- Dependencies at the HDL level are automatically identified and resolved by tokenizing file contents for valid primary design unit references across IP. The user is only responsible for specifying direct dependencies at the IP level.

## Examples

A fictitious organization, "Hyperspace Labs", exists for the purpose of demonstrating and learning how to leverage `orbit` in a real development setting. No identification with actual persons, places, buildings, and products is intended or should be inferred. 

The projects and code for Hyperspace Labs are walked through in the [tutorials](https://cdotrus.github.io/orbit/tutorials/tutorials.html) section.

The final code repositories for Hyperspace Labs are found [here](https://github.com/orgs/hyperspace-labs/repositories). 

## Installing

`orbit` has pre-built binaries for MacOS, Windows, and Ubuntu. See the [releases](https://github.com/cdotrus/orbit/releases) page to grab the latest release, or you can build from source with `cargo`. See the full installation instructions for complete details [here](https://cdotrus.github.io/orbit/1_starting/1_installing.html).

## Documentation

Read the [Book of Orbit](https://cdotrus.github.io/orbit/) for comprehensive documentation composed of tutorials, user guides, topic guides, references, and command manuals.

`orbit` provides commands for every stage of the development cycle, such as exploration, integration, and automation:

```
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip
    init            initialize an ip from an existing project
    view            display information about an ip
    read            navigate hdl design unit source code
    get             fetch an entity
    tree            view the dependency graph
    plan, p         prepare a target for processing
    build, b        execute a target
    run, r          prepare and execute a target
    launch          verify an upcoming release
    search          browse the ip catalog 
    download        request packages from the internet
    install         store an immutable reference to an ip
    env             print orbit environment information
    config          modify configuration values
    remove          uninstall an ip from the catalog

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --color <when>  coloring: auto, always, never
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.

```

## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).

## License

This project, which refers to all of the files and source code created and stored in this repository, is currently licensed under the open-source copyleft GPL-3.0 license. See [LICENSE](./LICENSE).