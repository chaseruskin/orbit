# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. 

- `ORBIT_HOME` - directory where orbit stores its data. By default it is `$HOME/.orbit` on Unix systems and `%USERPROFILE%/.orbit` on Windows systems.

- `ORBIT_CACHE` - directory where orbit caches installed IP. By default it is `$ORBIT_HOME/cache`.

- `NO_COLOR` - does not print colorized output when set to a value.

- `EDITOR` - chooses this value as the default text editor when no `core.editor` key is present in the config.toml.

## Runtime environment variables

Orbit also sets environment variables during runtime so a plugin has access to runtime information. 

- `ORBIT_PATH` - path to locate in-development IP. Unless explicitly set, Orbit will set this value to the path found as `core.path` set in config.toml.

- `ORBIT_BUILD_DIR` - directory to place the `blueprint.tsv` file relative to the current IP path. Default is `build` unless set as `core.build-dir` in config.toml.

- `ORBIT_TOP` - toplevel design unit identifier

- `ORBIT_BENCH` - toplevel design's testbench identifier


<!--Note about environment variables vs. settings file vs. arguments

precedence:
3. config file
2. env vars
1. command-line
-->