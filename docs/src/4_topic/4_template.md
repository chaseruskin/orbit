# Templates

Templates help you start a project faster.

Here is an example template stored in a configuration file:

_config.toml_
``` toml
[[template]]
alias = "main"
path = "template/"
summary = "standard project structure with rtl and sim folders"
ignore = [
    "extra/"
]
```

Templates are paths to a directory on your local filesystem that can be copied when creating a new IP with `orbit new`. 

Orbit automatically will omit copying a `.git` folder and a `Orbit.toml` file from the template's root directory. You can specify additional ignore rules with the template configuration's `ignore` entry.

## Variable Substitution

Templates support variable subsitution for more customized importing per project. Orbit searches for a double bracket notation `{{ }}` to signify a variable. Variables can exist in the filepath name or the file's contents.

### Example 
Assume the given variables defined by Orbit:
```
orbit.ip = ks-tech.rary.gates
orbit.ip.name = gates
orbit.user = Kepler
```

Then variable transformation would apply like so:

Original template filepath:   
```
rtl/{{orbit.ip.name}}.vhd
```

Imported filepath:
```
rtl/gates.vhd
```

Original template file contents:
``` vhdl
--! project: {{ orbit.ip }}
--! author: {{ orbit.user }}
entity {{ orbit.ip.name }} is

end entity;
```

Imported file contents:
``` vhdl
--! project: ks-tech.rary.gates
--! author: Kepler
entity gates is

end entity;
```

> __Note:__ Any variable that is not recognized by Orbit has its text left as-is and is not transformed.