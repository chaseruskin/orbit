# Variable Substitution

_Variable substitution_ is the process of injecting runtime information into specific locations. 

To have a variable be substituted with its value, use double opening curly brackets `{{` to denote the beginning of a variable key and double closing curly brackets `}}` to end the variable key. Whitespace is ignored around the variable key within the curly bracket sequences.

When Orbit gets a string in a context supported by variable substitution, it will parse the characters to check if a variable should be substituted. If it finds a valid variable with the same name, then it replaces everything from and within the curly bracket sequences with the variable's value. If it cannot find a valid variable that matches the name, it leaves that sequence of the string unmodified.

### Example

Consider a plugin with the following fileset defined.

``` toml
fileset.python-model = "{{ orbit.bench }}.py"
```

Given the context that the testbench name is "add_tb", then at runtime the fileset "PYTHON-MODEL" will resolve its file pattern to look for files that match "add_tb.py".

Now given a different context when the testbench name is called "mult_tb", then at runtime the same fileset "PYTHON-MODEL" will resolve its file pattern to look for files that match "mult_tb.py".

## Supported variables

Variable substitution is currently supported in the following contexts:

### Fileset patterns

The string pattern for a plugin's fileset configuration is allowed to contain any of the following substitution variables:
- `orbit.top`: The top-level design unit name.
- `orbit.bench`: The testbench design unit name.
- `orbit.env.*`: Any environment variables loaded from configuration files.

### Protocol arguments

The arguments set in a protocol's configuration are allowed to contain any of the following substitution variables:
- `orbit.queue`: The directory that Orbit expects the IP to be downloaded to.
- `orbit.ip.name`: The name of the IP being downloaded.
- `orbit.ip.version`: The version of the IP being downloaded.
- `orbit.ip.source.url`: The URL for the IP being downloaded.
- `orbit.ip.source.protocol`: The protocol specified by the IP being downloaded
- `orbit.ip.source.tag`: The tag (if provided) specified by the IP being downloaded.
- `orbit.env.*`: Any environment variables loaded from configuration files.

## Environment variable translation examples

A substitution variable key is the environment variable key but converted to lowercase with each "_" character replaced by a "." character.

Environment variables can be set in the configuration file like so.
``` toml
[env]
foo = "bar"
github-user = "cdotrus"
Yilinx_Path = "/Users/chase/fpga/bin/yilinx"
```

This configuration translates to the following variables:

| TOML `[env]` entry | Environment variable | Substitution variable |  
| - | - | - | 
| foo | ORBIT_ENV_FOO | orbit.env.foo |
| github-user | ORBIT_ENV_GITHUB_USER | orbit.env.github.user |  
| Yilinx_Path | ORBIT_ENV_YILINX_PATH | orbit.env.yilinx.path |
