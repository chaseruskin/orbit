# __orbit init__

## __NAME__

init - initialize an existing directory as an ip

## __SYNOPSIS__

```
orbit init [options] <ip>
```

## __DESCRIPTION__

This command will initialize an existing directory/project into a IP
recognized by Orbit. By default, the current working directory will be
initialized.
  
The `--path` option can be combined with `--git`. Then the project will be
cloned to the specified path. This path must either not exist or be empty.
If it is provided as a relative path, then it will be appended to the 
DEV_PATH. By default, `--git` will clone to the DEV_PATH under 
`$ORBIT_DEV_PATH/<vendor>/<library>/<name>`.

## __OPTIONS__

`<ip>`  
      The fully specified pkgid to name to the ip
 
`--git <repo>`  
      A git repository to clone
 
`--path <path>`  
      A filesystem destination for a git repository initialization

## __EXAMPLES__

```
orbit init ks-tech.rary.gates --git https://github.com/ks-tech/gates.git
```