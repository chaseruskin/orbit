### Priority

- [ ] add --verbose flag to orbit plan to print what units are set as top-level or testbench (or --quiet)

- [!] develop_from_lock_entry, denote in building ip graph from lockfile if an ip node is a development version and to not try to install from lock file, but place on dev path -> or skip entry

- [?] warn of blackboxes on launch

- [ ] create a logging enum (INFO, WARNING, ERROR)

- [ ] add example working environment for system testing demonstrating DST (three tree IP)

- [ ] warn on launch of unused dependencies in Orbit.toml

- [ ] `orbit sync --vendor <vendor>`: refresh vendors
    - [ ] (pull a vendor's remote repository first) push a vendor's remote repository if exists

- [ ] `orbit delete --ip <ip> --from <path> --tmp` remove ip from DEV_PATH

- [ ] build complete IP dependency tree from installs for `orbit probe` to show dependents

- [ ] change `orbit uninstall` default behavior to not look at DEV_PATH (reflects `orbit install` inability to look at DEV_PATH)
    - `orbit uninstall --dst --store --all --unused`
    - `--dst`: remove all dst instances
    - `--unused`: remove all unused cache values

    - should dst be written to user's `build/` folder?

- [ ]  `orbit develop` bring ip to development state (DEV_PATH)
    - `orbit edit project1 --all`
    - `orbit edit --ip project1 --mode open/path`
    - `orbit develop project1 --to <dest> --mode self|direct|all`

- [ ] default behavior of `orbit get` is ??? -> print initial comment block (--info?) -> leaning no due to `orbit read`

- [ ] @idea: add --mode option to `orbit new` ? -> create|open|path

- [ ] allow configurations to be set also as top level (along side their entities still possible too)

- [ ] @idea: use orbit install `--develop` to bring to DEV_PATH -> UPDATE: use `orbit edit` ? -> with `--mode pull`

### Backlog

- [ ] limit connecting edges to only neighboring projects in hdl graph building

- [ ] orbit run: plan and build in a single step

- [ ] add `--path <path>` as an option to change directories to run command there

- [ ] auto install from A if listed in dependencies on next `orbit plan`

- [ ] show dependents in `orbit probe` by generating entire ip graph over all of installations

- [ ] add a `dynamic` entry to lock file to allow audit to check dynamic checksum too.

- [ ] `orbit publish` publish a version to its registry (can also do --all for all versions)

- [ ] @idea: inject pre-launch script?

- [ ] @refactor: provide enum for IpManifest to differentiate between "pointer" and "source"?

- [ ] check the checksum on current ip to determine if plan needs to recompute the dependency resolution

- [x] update lock file on launch of new version (run a plan but do not save blueprint) -> NOT NEEDED. READ LOCK FILE WHEN INSTALLING, SO NEXT TIME YOU USE LOCK FILE (IF IN DEV), THEN ALL DEPS ARE ALREADY THERE AND WE REPRODUCE LOCKFILE NOW WITH NEW VERSION NUMBER.

- [ ] remove duplicate files from blueprint by taking latest file appearance (condense by filename using hashset to generate new graph of nodes) -> maybe infeasible/NBD (remove extra filepaths from blueprint by using hashset of `Rules`)

- [ ] run checksum to determine if needing to rerun plan during run command

### Bonus

- [ ] `orbit yank` add an additional git tag to a version to prevent it from being used in future designs as direct dependency (git tag: "orbit-yank")

- [o] improve hashing of repositories to include the remote if is available

- [?] fix lockfile error with dev versions by providing source as a path

- [ ] more powerful `get` command. Allow manipulation of signal/constant identifiers.
    - example: `--interface <pattern>` so `{{ orbit.instance }}_*_wire` -> `uX_data_wire` 
    - `--ports <pattern>`
    - `--generics <pattern>`
    - also allow this to be set in settings `config.toml`
    - Also return "crude"/"raw" form of entity declaration section to include comments

- [ ] have ability to report errors or turn them off in linting of VHDL (currently performing lossy conversion into tokens/symbols) -> improve lossy conversion to return Option<> rather than Error

- [ ] do string misspelling suggestions when requesting IP with PKGID and when requesting an entity with identifier

- [ ] verilog/systemverilog support

### Complete
