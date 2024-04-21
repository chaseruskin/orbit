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
  <a href="mailto:c.ruskin@ufl.edu?subject=Thanks%20for%20Orbit!">
    <img src="https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg" alt="say thanks">
  </a>
</div>
<br>

`orbit` is a package manager for Hardware Description Languages (HDL). 

Read the [Book of Orbit](https://cdotrus.github.io/orbit/) for complete documentation.

<br>

`orbit` manages your projects, called IPs, by handling the overhead for referencing, maintaining, and integrating your hardware description files:

```
lab2/
├─ Orbit.toml
├─ rtl/
│  ├─ reg.vhd
│  └─ top.vhd
└─ sim/
   └─ top_tb.vhd
```

<br>

Adding a simple TOML file `Orbit.toml` to a directory denotes that directory as an IP to `orbit`:

``` toml
[ip]
name = "lab2"
version = "1.0.0"

[dependencies]
lab1 = "1.0.4"
```

<br>

`orbit` generates VHDL code snippets able to be directly inserted into a new VHDL design for rapid reuse:
```
$ orbit get adder --ip lab1:1.0.4 --component --signals --instance
```
``` vhdl
component adder
  port (
    input1    : in  std_logic_vector(5 downto 0);
    input2    : in  std_logic_vector(5 downto 0);
    carry_in  : in  std_logic;
    sum       : out std_logic_vector(5 downto 0);
    carry_out : out std_logic
  );
end component;

signal input1    : std_logic_vector(5 downto 0);
signal input2    : std_logic_vector(5 downto 0);
signal carry_in  : std_logic;
signal sum       : std_logic_vector(5 downto 0);
signal carry_out : std_logic;

u_adder : adder
  port map (
    input1    => input1,
    input2    => input2,
    carry_in  => carry_in,
    sum       => sum,
    carry_out => carry_out
  );
```

<br>

`orbit` plans your build by generating a file list, called a blueprint, that contains a list of the required files for your given design in topologically-sorted order to act as an input to any backend toolchain:

```
VHDL-RTL	math	/users/chase/.orbit/cache/lab1-1.0.4-7f4d8c7812/rtl/fa.vhd
VHDL-RTL	math	/users/chase/.orbit/cache/lab1-1.0.4-7f4d8c7812/rtl/adder.vhd
VHDL-RTL	work	/users/chase/projects/lab2/rtl/reg.vhd
VHDL-RTL	work	/users/chase/projects/lab2/rtl/top.vhd
VHDL-SIM	work	/users/chase/projects/lab2/sim/top_tb.vhd
```

## Features

`orbit` has a lot more useful features relating to HDL package management and development:

- `orbit` is a frontend package manager, not a build system, so it allows users to define and automate their own workflows for building HDL designs.

- Linux, MacOS, and Windows are supported with no additional dependencies.

- Docker images of `orbit` are available for easy integration into new or existing CI/CD pipelines.

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
    show            print information about an ip
    read            inspect hdl design unit source code
    get             fetch an entity
    tree            view the dependency graph
    plan, p         generate a blueprint file
    build, b        execute a plugin
    launch          verify an upcoming release
    search          browse the ip catalog 
    download        request packages from the internet
    install         store an immutable reference to an ip
    env             print Orbit environment information
    config          modify configuration values
    uninstall       remove an ip from the catalog

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