# First Steps

Before diving into Orbit, there are a few things to do to make it a more useful experience.

## Initial Configurations

Orbit works well when you have a single path where you would like to keep all projects. This path is called the DEV_PATH. You can set this in the `config.toml` file under the user's `.orbit/` folder. You can also set this on the command-line.

```
$ orbit config --global --set core.path="path/to/keep/my/projects"
```

Orbit also can open these projects and various other files through certain commands, so you can set a default editor to open projects and files.

```
$ orbit config --global --set core.editor="code"
```

_config.toml_
``` toml
[core]
path = "path/to/keep/my/projects" # folder to store all projects under development
editor = "code" # executable/script that can open files/folders

```

## Seeking Help

Orbit is a package manager and development tool. With learning new tools there is always a learning curve. Orbit tries to make it less intimidating to use by offering help and information in a variety of ways. 

To see a list of common commands and options, just use `orbit` with no arguments.

To view quick summaries on commands, use `-h, --help` flags.

To view more detailed manual pages and information, use `orbit help`.

Complete documentation can be found on this current website.