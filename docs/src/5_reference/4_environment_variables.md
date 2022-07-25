# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. 

- `ORBIT_HOME` - directory where orbit stores its data. By default it is `$HOME/.orbit` on Unix systems and `%USERPROFILE%/.orbit` on Windows systems.

- `ORBIT_CACHE` - directory where orbit caches installed IP. By default it is `$ORBIT_HOME/cache`.

- `ORBIT_STORE` - directory where orbit saves IP git repositories. By default it is `$ORBIT_HOME/store`.

- `NO_COLOR` - does not print colorized output when set to a value.

- `EDITOR` - chooses this value as the default text editor when no `core.editor` key is present in the config.toml.

## Runtime environment variables

Orbit also sets environment variables during runtime so a plugin has access to runtime information. 

- `ORBIT_DEV_PATH` - path to locate mutable in-development IP. Unless explicitly set, Orbit will set this value to the path found as `core.path` set in config.toml.

- `ORBIT_BUILD_DIR` - directory to place the `blueprint.tsv` file relative to the current IP path. Default is `build` unless set as `core.build-dir` in config.toml.

- `ORBIT_IP_PATH` - path to the IP that is detected under the current working directory. If its not immediately detected at the current directory, it will continue to search the parent directory until it finds a `Orbit.toml` manifest file.

- `ORBIT_PLUGIN` - last referenced plugin from the planning phase

- `ORBIT_TOP` - toplevel design unit identifier

- `ORBIT_BENCH` - toplevel design's testbench identifier

- `ORBIT_IP` - current working directory's ip PKGID

- `ORBIT_IP_NAME` - name component of current working directory's ip PKGID

- `ORBIT_IP_LIBRARY` - library component of current working directory's ip PKGID

- `ORBIT_IP_VENDOR` - vendor component of current working directory's ip PKGID

- `ORBIT_IP_VERSION` - specific version of current working directory's ip

## Checking the environment

You can review the known environment variables within Orbit with `orbit env`.

<!--Note about environment variables vs. settings file vs. arguments

precedence:
3. config file
2. env vars
1. command-line
-->