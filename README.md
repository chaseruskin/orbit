# `orbit`

![pipeline](https://github.com/c-rus/orbit/actions/workflows/pipeline.yml/badge.svg) ![docs](https://github.com/c-rus/orbit/actions/workflows/docs.yml/badge.svg) [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

## The HDL package manager

Orbit is the package manager for Hardware Description Languages (HDL).

## Documentation

Read the [Book of Orbit](https://c-rus.github.io/orbit/).

```
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip
    init            initialize an ip from an existing project
    edit            open an ip in a text editor
    probe           access information about an ip
    get             fetch an entity
    tree            view the dependency graph
    plan            generate a blueprint file
    build, b        execute a plugin
    launch          release a new ip version
    search          browse the ip catalog
    install         store an immutable reference to an ip
    env             print Orbit environment information
    config          modify configuration values
    uninstall       remove an ip from the catalog

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.

```


## Contributing

See [CONTRIBUTING](./CONTRIBUTING.md).

## License

This project, which refers to all source code files created under this repository, is currently licensed under the open-source copyleft GPL-3.0 license. See [LICENSE](./LICENSE).