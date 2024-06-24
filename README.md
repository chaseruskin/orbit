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

Orbit is a package manager for hardware description languages (HDL).

### Live at the cutting edge of hardware design

The boom of AI and emerging workloads have shown just how fast new advancements can be made in models and algorithms. Today's hardware is no longer good enough to meet the immediate demands of tomorrow's innovations. It's time to build tomorrow's hardware, today. It's time to __live at the cutting edge of hardware design.__

### A package manager designed for minimizing technical debt 

As codebases scale and increase in complexity, it becomes of upmost importance to have the right system in place to efficiently manage the increasing number of resources. Without the right system, the codebase can be bogged down by _technical debt_, leaving you stuck in yesterday's designs.

However, using just any package management system does not guarantee that technical debt is minimized. Poorly-designed package managers will simply shift the technical debt to different resources, while a well-designed package manager will minimize the overall amount of technical debt. With minimal technical debt, you can bring up tomorrow's hardware today. Orbit is __a package manager designed for minimizing technical debt.__

### Free and open-source

Automated builds are available for Linux, MacOS, and Windows with no dependencies. Check out the [releases page](https://github.com/cdotrus/orbit/releases) for the latest version. Working on a different platform? No problem, building from source is easy with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), Rust's default package manager. Use docker? We have a docker image available too. See [Installing]((https://cdotrus.github.io/orbit/starting/installing.html)) for complete details.

For more information on getting started and how to use Orbit in your workflow, check out the [Book of Orbit](https://cdotrus.github.io/orbit/).

## Simple and intuitive to use

Orbit manages your projects by turning them into packages (referred to as ips) with the addition of a single manifest file: "Orbit.toml".

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

The "Orbit.toml" file is a TOML file that requires only a couple fields, such as the ip's `name` and `version`, to get setup. 

``` toml
[ip]
name = "cpu"
version = "1.0.0"

[dependencies]
gates = "2.0.0"
```

## Low effort integration

To encourage code reuse and faster development cycles, Orbit includes HDL-specific commands to integrate designs across ips. For example, Orbit can display HDL code snippets of other known designs to be used in the current working ip.

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

## Flexibility in use

Since Orbit only focuses on efficiently managing the HDL source code and minimizing its associated technical debt, users have the power to define their own back end processes. This is achieved with Orbit passing off a file that lists the topologically-sorted order of HDL source code files for a user's back end process to accept. 

```
VHDL-RTL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/nand_gate.vhd
VHDL-RTL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/and_gate.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/datapath.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/ctrl.vhd
VHDL-RTL	work	/users/chase/projects/cpu/rtl/top.vhd
VHDL-SIM	work	/users/chase/projects/cpu/sim/top_tb.vhd
```

Write a script to accept Orbit's output file for whatever EDA tools you prefer once, and use it across all future ips.  

## Features

Orbit has a lot more useful features relating to HDL package management and development:

- Orbit only focuses on source code management, allowing users to define and automate their own HDL back end processes.

- Linux, MacOS, and Windows are supported with no additional dependencies.

- Docker images of Orbit are available for easy integration into new or existing CI/CD pipelines.

- A GitHub Action is available to install orbit for GitHub workflows using [`cdotrus/setup-orbit`](https://github.com/cdotrus/setup-orbit.git).

- Reproducible builds are achieved with checksums and automatic handling of a lockfile `Orbit.lock`. 

- Namespace collisions, a problem inherent to VHDL and not resolved in many backend tools, is solved through a custom algorithm called [_dynamic symbol transformation_](https://cdotrus.github.io/orbit/topic/dst.html).

- Multiple versions of the same entity (or more broadly, entities given the same identifier) are allowed in the same build under [two simple constraints](https://cdotrus.github.io/orbit/topic/dst.html#limitations).

- Navigate HDL source code efficiently to read its inline documentation and visit its implementation through Orbit's ability to locate HDL code segments

- Produce VHDL code snippets with a single command to properly instantiate entities within a new design.

- Quickly search through your IP catalog by filtering based on keywords, catalog status, and name.

- Avoid being locked into a specific vendor's tooling through Orbit's common interface with a flexible build command to adapt to any workflow.
  
- Orbit is version control system (VCS) agnostic through defining custom protocols for fetching IP. Continue to use your current VCS (or none).

- Minimal user upkeep is required to maintain a manifest file `Orbit.toml` that identifies an IP, its metadata, and any dependencies from your catalog.

- View the current design's tree hierarchy at an HDL entity level or at an IP level.

- Implement custom scripted workflows through a plugin system.

- Specify additional supportive files to be passed to your backend workflows with filesets.

- Dependencies at the HDL level are automatically identified and resolved by tokenizing file contents for valid primary design unit references across IP. The user is only responsible for specifying direct dependencies at the IP level.

## Examples

A fictitious organization, "Hyperspace Labs", exists for the purpose of demonstrating and learning how to leverage Orbit in a real development setting. No identification with actual persons, places, buildings, and products is intended or should be inferred. 

The projects and code for Hyperspace Labs are walked through in the [tutorials](https://cdotrus.github.io/orbit/tutorials/tutorials.html) section.

The final code repositories for Hyperspace Labs are found [here](https://github.com/orgs/hyperspace-labs/repositories). 

## Installing

Orbit has pre-built binaries for MacOS, Windows, and Ubuntu. See the [releases](https://github.com/cdotrus/orbit/releases) page to grab the latest release, or you can build from source with `cargo`. See the full installation instructions for complete details [here](https://cdotrus.github.io/orbit/starting/installing.html).

## Documentation

Read the [Book of Orbit](https://cdotrus.github.io/orbit/) for comprehensive documentation composed of tutorials, user guides, topic guides, references, and command manuals.

Orbit provides commands for every stage of the development cycle, such as exploration, integration, and automation:

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