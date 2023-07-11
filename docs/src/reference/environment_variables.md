# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. 

- `ORBIT_HOME` - directory where orbit stores its data. By default it is `$HOME/.orbit` on Unix systems and `%USERPROFILE%/.orbit` on Windows systems.

- `ORBIT_CACHE` - directory where orbit caches installed IP. By default it is `$ORBIT_HOME/cache`.

- `ORBIT_DOWNLOADS` - directory where orbit saves archived snapshots of IP at a particular version.

- `NO_COLOR` - does not print colorized output when set to a value.

- `EDITOR` - chooses this value as the default text editor when no `core.editor` key is present in the config.toml.

- `ORBIT_WIN_LITERAL_CMD` - disables default behavior of checking for programs ending with .exe then .bat when a program name without extension is not found on a windows operating system

## Runtime environment variables

Orbit also sets environment variables during runtime so a plugin has access to runtime information. 

- `ORBIT_BUILD_DIR` - directory to place the `blueprint.tsv` file relative to the current IP path. Default is `build`.

- `ORBIT_IP_PATH` - path to the IP that is detected under the current working directory. If its not immediately detected at the current directory, it will continue to search the parent directory until it finds a `Orbit.toml` manifest file.

- `ORBIT_PLUGIN` - last referenced plugin from the planning phase

- `ORBIT_TOP` - toplevel design unit identifier

- `ORBIT_BENCH` - toplevel design's testbench identifier

- `ORBIT_IP_NAME` - name field of the manifest for the IP package

- `ORBIT_IP_LIBRARY` - optional HDL library defined in the manifest for the IP package

- `ORBIT_IP_VERSION` - specific version of current working directory's IP

- `ORBIT_BLUEPRINT` - the filename for the blueprint: `blueprint.tsv`

## Checking the environment

You can review the known environment variables within Orbit with `orbit env`.

<!--Note about environment variables vs. settings file vs. arguments

precedence:
3. config file
2. env vars
1. command-line
-->