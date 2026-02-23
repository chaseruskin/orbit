# __orbit clean__

## __NAME__

clean - remove the target directory

## __SYNOPSIS__

```
orbit clean [options]
```

## __DESCRIPTION__

Remove artifacts that Orbit has generated in the past.

By default, this command deletes all directories found within the current
project's target directory. To select a single target's directory for removal,
use the `--target` option.

## __OPTIONS__

`--doc`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Whether or not to clean just the documentation directory

`--target, -t <name>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Target output directory to just remove

`--target-dir <dir>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; The relative directory where the targets exist

## __EXAMPLES__

```
orbit clean
orbit clean --target modelsim
```

