<!--
This changelog follows a very particular format. 

Only the title 'changelog' may begin with 1 pound symbol '#'. 

Every version partition must begin with 2 pound symbols '##'. 

Any section under a version must begin wtih 3 pound symbols '###'. 

This is important for the auto-changelog extraction occuring during the CI/CD 
pipeline to list only the current verion's changes with every release. 
-->

# Changelog

## 0.1.1

### Bug Fixes

- `--upgrade` flag now properly searches for correct binary dependent on operating system during zip extraction

## 0.1.0

### Features

- implements basis for `new`, `edit`, `tree`, `plan`, `build`, `launch`, `search`, and `install` commands

- view current design heirarchy using `tree` command

- allows an option to be accepted multiple times on the command-line (example: `--filesets`)

- allows filesets to be created/overriden on command-line for `plan` command

- verifies `build` and `plan` commands are called from an "IP-directory-sensitive" path

- prevents IPs from becoming nested within each other's path in `new` command

- adds `build` command to execute an external process within orbit

- adds `plan` command to generate a blueprint.tsv file in a build output directory

- adds support for the toml configuration file format to store key value pairs

- creates home folder at ~/.orbit if `ORBIT_HOME` enviornment variable is not set

- adds `--upgrade` flag for self-updating binary with latest Github release for user's targeted OS and architecture

- adds command-line interface with helpful misspelling suggestions and argument input validation