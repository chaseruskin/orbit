<!--
This changelog follows a very particular format. 

Only the title 'changelog' may begin with 1 pound symbol '#'. 

Every version partition must begin with 2 pound symbols '##'. 

Any section under a version must begin wtih 3 pound symbols '###'. 

This is important for the auto-changelog extraction occuring during the CI/CD 
pipeline to list only the current verion's changes with every release. 
-->

# Changelog

## 0.1.6

### Fixes
- copying directories now goes to correct level for `init` and `install` commands
- fixes issue with install failing on windows due to files being used by other processes

### Changes
- search command uses `--install, -i` rather than `--cache, -c` to show installed IP
- displays clearer error message if an IP is already installed

## 0.1.5

### Features
- new command: `init`- initialize existing projects as orbit IP

### Fixes
- users can now no longer create an IP with same pkgid as an already known IP from any level (dev, cache, available)

### Documentation
- adds repository key to orbit.toml page
- writes vendor page and adds blank user guides
- writes `init` man page

## 0.1.4

### Fixes
- improves vhdl symbol detection for parsing functions and procedures 
- resolves path issue for `get` command on current ip
- properly checks config.toml file with 'core' table before searching it
- ensures `core.editor` exists for edit command

### Documentation
- adds topics for ip and manifest

## 0.1.3

### Features
- enhances plugin display list using `orbit plan --list`
- supports graph building with in-order references to packages during `plan` command
- adds more progress messages to `--upgrade` action
- reads ip found in cache during `search` command

### Bug Fixes
- running `plan` with no `--top` and no `--bench` while having it auto-detect only a top does not result in an error
- nicer error mesage for identifier not found during `tree` command

## 0.1.2

### Features
- an installer program is included with the orbit project when downloading a precompiled binary

### Fixes
- windows-style endings (`\r\n`) supported when given interactive prompts

### Documentation
- adds a simple README included with the orbit project for distribution

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