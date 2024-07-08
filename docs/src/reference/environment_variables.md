# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. 

- `ORBIT_HOME` - directory where orbit stores its data. By default it is `$HOME/.orbit` on Unix systems and `%USERPROFILE%/.orbit` on Windows systems.

- `ORBIT_CACHE` - directory where orbit caches installed ip. By default it is `$ORBIT_HOME/cache`.

- `ORBIT_ARCHIVE` - directory where orbit saves archived snapshots of ip at a particular version. By default it is `$ORBIT_HOME/archive`.

- `ORBIT_CHANNELS` - directory where orbit checks for channels that point to available ip. By default it is `$ORBIT_HOME/channels`.

- `NO_COLOR` - does not print colorized output when set to a value.

- `EDITOR` - chooses this value as the default text editor when no `core.editor` key is present in the config.toml.

- `ORBIT_WIN_LITERAL_CMD` - disables default behavior of checking for programs ending with .exe then .bat when a program name without extension is not found on a windows operating system

## Runtime environment variables

Orbit also sets environment variables during runtime so a plugin has access to runtime information. 

- `ORBIT_TARGET_DIR` - directory to perform the build process relative to the current ip path. Default is `target`.

- `ORBIT_IP_PATH` - path to the ip that is detected under the current working directory. If its not immediately detected at the current directory, it will continue to search the parent directory until it finds a `Orbit.toml` manifest file.

- `ORBIT_TARGET` - selected target to plan and execute

- `ORBIT_TOP` - top level design unit identifier

- `ORBIT_BENCH` - the testbench identifier

- `ORBIT_DUT` - the device under test's identifier

- `ORBIT_IP_NAME` - name field of the manifest for the ip package

- `ORBIT_IP_LIBRARY` - optional HDL library defined in the manifest for the ip

- `ORBIT_IP_VERSION` - specific version of current working directory's ip

- `ORBIT_BLUEPRINT` - the filename for the blueprint created during the planning stage

- `ORBIT_OUTPUT_PATH` - path to the selected target's build process working directory

## Checking the environment

You can review the known environment variables within Orbit with `orbit env`.

<!--Note about environment variables vs. settings file vs. arguments

precedence:
3. config file
2. env vars
1. command-line
-->