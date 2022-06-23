# Create Plugins

A plugin is a command to execute a custom workflow during development.

Plugins are defined in the `config.toml` file.

Let's walk through an example plugin.

## Example: xsim

To utilize backend tools for HDL development, users can set up plugins to be used
across projects. A plugin is called by its _alias_. The _command_ is the first argument called in the process following with all the arguments in _args_, and then any additionally set arguments set on the command-line.  

_xsim plugin configuration in config.toml_
``` toml
[[plugin]]
alias = "xsim"
command = "python"
args = ["plug/orbit-xsim.py"]
summary = "basic toolflow for xsim executable"
fileset.xsim-tcl = "*_xsim.tcl"
fileset.xsim-wcfg = "*.wcfg"

# ...
```

What is actually ran underneath Orbit when calling "xsim" during build:

```
$ python plug/orbit-xsim.py
```

Note the filepath is relative to the `config.toml` file's location, if an argument is a relative path, Orbit will resolve it before running the command.

Filesets can be defined to help prepare what files you will need during the build. The planning phase will collect files that glob-style match the given patterns and place them in the blueprint.tsv file for building.