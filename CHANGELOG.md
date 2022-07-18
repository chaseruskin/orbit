<!--
This changelog follows a very particular format. 

Only the title 'changelog' may begin with 1 pound symbol '#'. 

Every version partition must begin with 2 pound symbols '##'. 

Any section under a version must begin wtih 3 pound symbols '###'. 

This is important for the auto-changelog extraction occuring during the CI/CD 
pipeline to list only the current verion's changes with every release. 

Add `- unreleased` along the next future version to prevent CI/CD from triggering release mechanism.
-->

# Changelog

## 0.3.12

### Features
- adds `--disable-ssh` flag to `orbit install`

### Fixes
- properly computes checksum during install and reverts to original cwd

## 0.3.11

### Features
- adds `--disable-ssh` flag to `orbit plan` to convert ssh connections to ssh in lock file
- displays direct dependencies in `orbit probe`
- adds `--ip` flag to `orbit tree` to view IP-level dependency tree
- implements basic vendor registry support in `config.toml`
- ip can be installed from available state

### Documentation
- updates configuration page for vendor entry
- updates vendors page

## 0.3.10

### Features
- adds `orbit uninstall` command to remove ip from catalog (development and installations)
- adds `--name` option to `orbit get` in setting the HDL instance's identifier

### Fixes
- verifies `--name` can only be paired with `--instance` on `orbit get`

### Documentation
- adds content to user guide pages

## 0.3.9

### Fixes
- properly writes toml comment to initial config.toml

## 0.3.8

### Changes
- adds computed and expected checksums during a failed upgrade with `--upgrade`

## 0.3.7

### Features
- `orbit env` displays environment variabels from config.toml
- supports variable replacement in fileset patterns during `orbit plan`
- displays architectures with --architecture flag in `orbit get`
- filters search results by accepted pkgid ip arg for `orbit search`

### Fixes
- resolves bugs when using --git and --path together for `orbit init`
- enhances array formatting when appending to include key using `orbit config`

### Documentation
- adds blueprint topic page
- adds entries to glossary
- adds disclaimer about license
- grammar revisions
- adds orbit.lock page
- begins first tutorials page

## 0.3.6

### Features
- prevents launch when dependency is from dev path
- a lockfile `Orbit.lock` is written during plan command
- reads `Orbit.lock` when current ip's lock entry matches manifest data to ensure all dependencies are pulled in

### Fixes
- url in blank `Orbit.toml` files points to the correct Orbit.toml help page

## 0.3.5

### Fixes
- Orbit.toml is no longer needed on git main branch of an ip to install

## 0.3.4

### Fixes
- installing with git works without having main branch being Orbit ip
- reading deps from Orbit.toml now correctly travels to all dependencies

## 0.3.3

### Features
- new config.toml entry: `core.user` (used for template variable `{{ orbit.user }}` )
- new config.toml entry: `core.date-fmt` (used for template variable `{{ orbit.date }}` )
- launch command now installs by default
- initial support for external dependencies in plan and tree commands
- tree command now supports `--format long` option to display full pkgid with every entity

### Fixes
- config --set now properly overwrites existing value
- entity direct instantiation now contains missing 'entity' keyword
- better error handling for toml deserialization

### Documentation
- updates config.toml entry references for date-fmt and user

## 0.3.2

### Fixes
- fixes edge-case checksum bugs

## 0.3.1

### Features
- adds --verbose flag to build command
- stores repositories into store/ to be used to checkout specific installation versions into cache

### Documentation
- revises develop ip page
- writes developing ip page
- writes template page
- updates initial setup and new command pages
- adds config man page to index
- updates configuration page

## 0.3.0

### Features
- `config` command: modify some configuration values directly from the command-line
- supports local config.toml files in current working ip
- templates: reusable directories with variable substitution support to import boilerplate files for new projects
- adds `--list` flag to new command to view usable templates

### Documentation
- updates initial setup and new command pages
- adds config man page to index
- updates configuration page

## 0.2.2

### Features
- the home config file (located at ORBIT_HOME) can now include other config files on your machine with the `include` key (value is a a list of strings)

## 0.2.1

### Features
- print environment information with `env` command
- allows `b` as shortcut to `build` command
- `plan` command now saves the plugin it was called with (if any). In this case the next future calls to `build` can opt out of specifying a plugin to default to the one used during the previous `plan`.

### Changes
- install accepts multiple methods to install from (--path, --git, --ip)
- `launch` checks if a remote repository is entered in the manifest if the git repository for that ip has one

### Fixes
- `probe --units` now displays units in alphabetical order
- `--list` for printing plugins now adds in newlines between every plugin

## 0.2.0

### Features
- view component declarations, signal declarations, and instantiation code using the `get` command
- version can be specified for `probe` command

### Changes
- `query` command is now the `probe` command- still functions the same
- improves interface with using the `install` command and entering arguments

### Fixes
- verifies Orbit.toml is not ignored by git
- adds nicer error message with suggestion when no installation version is found

### Documentation
- adds -m for launch and fixes man page formatting

## 0.1.8

### Changes
- improves ip manifest detection in filesystem traversal
- improves checksum generation and stores a local checksum file in its cache slot for internal reference

### Fixes
- `init` command works properly using existing paths and current working dir
- `build` command properly displays plugins with `--list`

### Documentation
- `init` manual accurately reflects its action

## 0.1.7

### Changes
- `launch` now checks against remote upstream branch to verify its in sync before approving
- `build` is forced to either have a plugin or command passed into cli to run

### Documentation
- adds first steps and plugins pages
- adds query command manual page

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