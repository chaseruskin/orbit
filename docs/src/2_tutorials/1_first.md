# First Steps

Before diving into Orbit, there are a few things to do to make it a more useful experience.

## Initial Configurations

Orbit works well when you have a single path where you would like to keep all projects. This path is called the DEV_PATH. You can set this in the `config.toml` file under the user's `.orbit/` folder. 

Orbit also can open these projects and various other files through certain commands, so you can set a default editor to open projects and files.

_config.toml_
``` toml
[core]
path = "path/to/folder"         # folder to store all projects under development
editor = "path/to/executable"   # executable/script that can open files/folders

```

## Seeking Help

Orbit is a package manager and development tool. With learning new tools there is always a learning curve. Orbit tries to make it less intimidating to use by offering help and information in a variety of ways. To see a list of common commands and options, just run `orbit` with no arguments.
```
$ orbit
Orbit is a tool for hdl package management.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip
    init            initialize an ip from an existing project
    edit            open an ip in a text editor
    get             fetch an entity
    tree            view the dependency graph
    plan            generate a blueprint file
    build           execute a plugin
    launch          release a new ip version
    search          browse the ip catalog
    install         store an immutable reference to an ip

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
```

The `-h, --help` flag is useful to read the quick help summary of a subcommand. If you need more information, the `help` command displays manual pages about subcommands and topics.

```
$ orbit help plan
NAME
    plan - generate a blueprint file

SYNOPSIS
    orbit plan [options]

DESCRIPTION
    This command will set up the current ip for build processes. It will collect
    all necessary files according to their defined fileset into the
    blueprint.tsv file.

    By default, the top level unit and testbench are auto-detected according to
    the current design heirarchy. If there is ambiguity, it will ask the user to
    select one of the possibilities when not set as options.

    The top level unit and top level testbench will be stored in a .env file to
    be set during any following calls to the 'build' command.

OPTIONS
    --top <unit>
          The top level entity to explicitly define

    --bench <tb>
          The top level testbench to explicitly define

    --plugin <alias>
          A plugin to refer to gather its declared filesets

    --build-dir <dir>
          The relative directory to place the blueprint.tsv file

    --filset <key=glob>...
          A glob-style pattern identified by a name to add into the blueprint

    --clean
          Removes all files from the build directory before planning

    --list
          Display all available plugins and exit

    --all
          Ignore any design hierarchy and include all hdl files
          
EXAMPLES
    orbit plan --top top_level --fileset PIN-PLAN="*.board"
```