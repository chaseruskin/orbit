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
  
If the `--git` option is combined with `--path`, then the project will be
cloned to the specified path. This path must either not exist or be empty. 

## __OPTIONS__

`<ip>`  
      The fully specified pkgid to name to the ip
 
`--git <repo>`  
      A git repository to clone
 
`--path <path>`  
      A filesystem destination to initialize the ip

## __EXAMPLES__

```
orbit init ks-tech.rary.gates --git https://github.com/ks-tech/gates.git
```