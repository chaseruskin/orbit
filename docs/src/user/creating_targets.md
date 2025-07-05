# Creating Targets

A target is a user-defined backend process. This section provides steps for ways to configure a target as well as write the script for a target.

### Assumptions

The steps below assume (and encourage) your targets to be written in Python, although targets can be written in any language and called by any command.

If a step mentions the "target script", this corresponds to the script which will be called when performing the execution stage of the build process.

If a step mentions the "target configuration", this corresponds to the entry for the target that has been defined in an Orbit configuration file (config.toml).

### Guides
- [Configuring a target](#configuring-a-target)
- [Reading a blueprint](#reading-a-blueprint)
- [Handling built-in filesets](#handling-built-in-filesets)
- [Adding a custom fileset](#adding-a-custom-fileset)
- [Adding a custom recursive fileset](#adding-a-custom-recursive-fileset)
- [Handling a custom fileset](#handling-a-custom-fileset)
- [Handling additional arguments](#handling-additional-arguments)

## Configuring a target

This guide walks through how to add a target to be recognized by Orbit. In this guide, our target is called `tar`.

1. Open an Orbit configuration file.
   
2. Make a new entry in the `[[target]]` array:
``` toml
[[target]]
name = "tar"
description = "My target description"
command = ["python", "tar.py"]
```

Calling the `tar` target at the start of a build process will now execute the command `python tar.py` for the execution stage.

Although the execution stage spawns the subprocess in the target's output directoy, any relative file paths defined in the `command` field are relative to the configuration file that defines them, if and only if it is a path that exists in relation to the configuration file's parent directory.

## Reading a blueprint

This guide walks through how to capture the data written to the blueprint from the planning stage in a target script.

1. Open the target script.

2. Add this code snippet to read the default plan (tab-separated value files) and process the data into the `entries` variable:
``` python
import os

entries = []
with open(os.environ['ORBIT_BLUEPRINT']) as bp:
    for line in bp.readlines():
        entries += [tuple(line.strip().split('\t'))]
```

## Handling built-in filesets

The blueprint always collects Verilog, SystemVerilog, and VHDL files. These files are encouraged to always be handled by the target script, whether it be for compilation, synthesis, linting, etc.

1. Open the target script.

2. Read the blueprint into a variable (`entries`) storing the blueprint's data as a list of 3-element (fileset, library, filepath) tuples.

3. Add this code snippet to do some pseudo-processing of the built-in filesets collected from the planning stage:
``` python
for entry in entries:
    # Break out the 3-element tuple into its respective components
    fileset, library, filepath = entry
    # Condition based on the entry's fileset
    if fileset == 'VHDL':
        print('TODO: handle vhdl file '+filepath+' into library '+library)
    elif fileset == 'VLOG':
        print('TODO: handle verilog file '+filepath+' into library '+library)
    elif fileset == 'SYSV':
        print('TODO: handle systemverilog file '+filepath+' into library '+library)
```

This will create placeholders ("TODO") print statements in the code sections where Verilog, SystemVerilog, and VHDL files can be processed by the target script.

## Adding a custom fileset

This guide walks through how to modify a target's configuration to accept additional files to be collected into the blueprint from only the current project during the planning stage.

1. Open the target configuration.

2. Add the fileset and its glob-style pattern to your target's entry:
``` toml
[[target]]
name = "tar"
description = "My target description"
command = ["python", "tar.py"]
# Add a custom fileset
fileset.A-SET = "*.txt"
```

The fileset also supports being able to store more than one pattern. Use the inline table syntax to specify an array of patterns for the fileset's `pattern` field:
``` toml
fileset.B-SET = { patterns = ["*.b", "*.b2"] }
```

If a file matches a least one of defined file patterns, then it will be added under that fileset.

## Adding a custom recursive fileset

This guide walks through how to modify a target's configuration to accept additional files to be collected into the blueprint from all projects in the dependency graph during the planning stage.

1. Open the target configuration.

2. Add the fileset, its glob-style pattern, and the `recursive` entry true to your target's entry:
``` toml
[[target]]
name = "tar"
description = "My target description"
command = ["python", "tar.py"]
# Add a custom recursive fileset
fileset.C-SET = { patterns = ["*.c"], recursive = true }
```

## Handling a custom fileset

Adding a custom fileset in the target's configuration is not enough to support custom files; the target script must also have support for handling the custom filesets that are collected into the blueprint from the planning stage.

1. Open the target script.

2. Read the blueprint into a variable (`entries`) storing the blueprint's data as a list of 3-element (fileset, library, filepath) tuples.

3. Add this code snippet to pseudo-process the custom files collected from the planning stage for the `A-SET` filset:
``` python
for entry in entries:
    # Break out the 3-element tuple into its respective components
    fileset, library, filepath = entry
    # Condition based on the entry's fileset
    if fileset == 'A-SET':
        print('TODO: handle text file '+filepath+' under the custom fileset '+fileset)
```

This will create placeholders ("TODO") print statements in the code sections where any file that was found to match the `A-SET` file pattern can be processed by the target script.

## Handling additional arguments

Sometimes arguments need to be passed to your target on the command line. Arguments found after `--` flag for `orbit build` or `orbit test` will be appended to the list of arguments already defined in the target's configuration.

1. Add this code snippet to check if `--verbose` or `--help` was an argument to the target script:
``` python
import sys

is_verbose = False
show_help = False
# Iterate through all arguments after the script name
for arg in sys.argv[1:]:
    if arg == '--verbose':
        is_verbose = True
    elif arg == '--help':
        show_help = True
    else:
        print('TODO: handle target argument '+arg)
```

2. Pass arguments after the `--` when starting the build process for the target:
```
orbit build --target tar -- option1 --verbose --help 
```

For a more robust solution to handling additional arguments, it is recommended to use an existing argument parsing library such as [`argparse`](https://docs.python.org/3/library/argparse.html).