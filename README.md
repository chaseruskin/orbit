# Orbit

[![Pipeline](https://github.com/cdotrus/orbit/workflows/pipeline/badge.svg)](https://github.com/cdotrus/orbit/actions) 
[![Documentation](https://github.com/cdotrus/orbit/actions/workflows/docs.yml/badge.svg)](https://cdotrus.github.io/orbit) 
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) 
[![Docker Hub](https://img.shields.io/badge/dockerhub-images-important.svg?logo=docker)](https://hub.docker.com/repository/docker/cdotrus/orbit/general) 
[![Downloads](https://img.shields.io/github/downloads/cdotrus/orbit/total.svg)](https://github.com/cdotrus/orbit/releases)

Orbit is an agile package manager for hardware description languages (HDL).

### Live at the cutting edge of hardware design

The boom of AI and emerging workloads have shown just how fast new advancements can be made in models and algorithms. Today's hardware is no longer good enough to meet the immediate demands of tomorrow's innovations; today's hardware must shift to a more agile development approach. It's time to build tomorrow's hardware, today. It's time to __live at the cutting edge of hardware design.__

### An agile package manager designed to minimize technical debt 

As codebases scale and increase in complexity, it becomes of upmost importance to have the right system in place to efficiently manage the increasing number of resources. Without the right system, the codebase can become bogged down by _technical debt_, leaving you stuck in yesterday's designs.

However, using just any package management system does not guarantee that technical debt is minimized. Poorly-designed package managers will simply shift the technical debt to different resources, while a well-designed package manager will minimize the overall amount of technical debt. With minimal technical debt, you can bring up tomorrow's hardware today. Orbit is __an agile package manager designed to minimize technical debt.__

### Free and open source

Orbit is available free to use and open source to encourage adoption, contribution, and integration among the hardware community. We rely on the open source community for feedback and new ideas, and are very grateful to our sponsors who keep this project going.

Prebuilt binaries are available for Linux, MacOS, and Windows with no dependencies. Visit the [releases page](https://github.com/cdotrus/orbit/releases) for the latest version. Working on a different platform? No problem, building from source is easy with [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html), Rust's default package manager. Use docker? We have [docker images](https://hub.docker.com/repository/docker/cdotrus/orbit/general) available too. See [Installing](https://cdotrus.github.io/orbit/starting/installing.html) for complete details.

For more information on getting started and how to use Orbit in your workflow, check out the [Book of Orbit](https://cdotrus.github.io/orbit/).

## Simple and intuitive to use

Orbit manages your project by turning it into a package (referred to as an ip) with the addition of a single file: "Orbit.toml."

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

The "Orbit.toml" file is a simple file written in TOML syntax that requires only a couple fields, such as the ip's `name` and `version`, to get setup. 

``` toml
[ip]
name = "cpu"
version = "1.0.0"

[dependencies]
gates = "2.0.0"
```

## Low effort integration

To encourage code reuse and faster development cycles, Orbit includes HDL-specific commands to integrate designs across ips. For example, Orbit can display HDL code snippets of other known design units to be instantiated within the current working ip.

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
VHDL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/nand_gate.vhd
VHDL	gates	/users/chase/.orbit/cache/gates-2.0.0-7f4d8c7812/rtl/and_gate.vhd
VHDL	work	/users/chase/projects/cpu/rtl/datapath.vhd
VHDL	work	/users/chase/projects/cpu/rtl/ctrl.vhd
VHDL	work	/users/chase/projects/cpu/rtl/top.vhd
VHDL	work	/users/chase/projects/cpu/sim/top_tb.vhd
```

Write a script to accept Orbit's output file for whatever EDA tools you prefer once, and use it across all future ips.  

## Highlights

What makes Orbit an agile package manager for HDLs? Here's some of its key features:

- Orbit acts as the intermediary between your source code and back end EDA tools, automating the upkeep process to minimize technical debt as your codebase evolves over time

- Reproduce results across any environment with Orbit through its automatic handling of lockfiles and checksums

- Overcome namespace collisions, a problem inherent to VHDL and Verilog, through a custom aglorithm that dynamically transforms conflicting design names called [_dynamic symbol transformation_](https://cdotrus.github.io/orbit/topic/dst.html)

- Because of dynamic symbol transformation, multiple versions of the same design unit (or more broadly, design units given the same identifier) are allowed in the same build under [two simple constraints](https://cdotrus.github.io/orbit/topic/dst.html#limitations)

- Quickly navigate through HDL source code to read its inline documentation and review a design unit's implementation with Orbit's ability to jump to and display HDL code segments

- Integrate existing design units across projects faster than ever with Orbit's ability to display valid HDL code snippets for design unit instantiation

- Explore your evolving codebase to identify the projects you need next with Orbit's ability to quickly search through known ip by filtering based on keywords, status, and name

- Keep your source code independent of vendor tools and avoid vendor lock-in with Orbit's vendor-agnostic interface to back end EDA tools

- Continue to use your preferred version control system (or none) due to Orbit's flexible approach to being version control system agnostic

- Review high-level design unit circuit tree hierarchies at the HDL level or ip level

- No longer worry about manually organizing a design unit's order of dependencies with Orbit's built-in ability to tokenize HDL source code and automatically identify valid references to other design units

- Linux, MacOS, and Windows are fully supported with zero dependencies

- Docker images and GitHub Actions are available to support CI/CD workflows

- Manifest files that mark a project as an ip only require a few user-defined fields to get setup

- Write a target for your preferred EDA tools once, and reuse across projects with Orbit's support for configuration files

And these are only a few of Orbit's features! Download Orbit and read its documentation today to discover everything Orbit provides as an agile package manager for HDLs. 

## Installing

Orbit has prebuilt binaries for MacOS, Windows, and Linux. See the [releases page](https://github.com/cdotrus/orbit/releases) to download the latest version, or build from source using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)- Rust's default package manager. See [Installing](https://cdotrus.github.io/orbit/starting/installing.html) for more details on getting Orbit up and running.

## Examples

A fictitious organization, "Hyperspace Labs", exists for the purpose of demonstrating and learning how to leverage Orbit in a real development setting. No identification with actual persons, places, buildings, and products is intended or should be inferred. 

The projects and code for Hyperspace Labs are walked through in the [tutorials](https://cdotrus.github.io/orbit/tutorials/tutorials.html) section.

The final code repositories for Hyperspace Labs are found [here](https://github.com/orgs/hyperspace-labs/repositories). 

## Documentation

Read the [Book of Orbit](https://cdotrus.github.io/orbit/) for comprehensive documentation composed of tutorials, user guides, topic guides, references, and command manuals.

Orbit brings an agile approach to hardware development that minimizes technical debt through its available commands related to ip exploration, integration, and automation:
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