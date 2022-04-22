# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. 

- `ORBIT_HOME` - directory where orbit stores its data. By default it is `$HOME/.orbit`.

- `ORBIT_CACHE` - directory where orbit caches installed IP. By default it is `$ORBIT_HOME/cache`.

- `NO_COLOR` - does not print colorized output when set to a value.

- `EDITOR` - chooses this value as the default text editor when no `core.editor` key is present in the config.toml

<!--
- `ORBIT_BUILD_DIR` - directory to locate the `blueprint.tsv` file within the current IP. By default is called `build`.
-->
### Runtime environment variables

Orbit also sets environment variables during runtime so a plugin has access to runtime information. 

<!-- 
- `ORBIT_TOP` - the toplevel design unit identifier

- `ORBIT_BENCH` - the toplevel design's testbench identifier
-->


<!--Note about environment variables vs. settings file vs. arguments

environment variables: only to change across systems (infrequent)

settings file: 

 -->