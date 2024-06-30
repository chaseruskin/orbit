# __orbit test__

## __NAME__

test - run a test

## __SYNOPSIS__

```
orbit test [options] [--] [args]...
```

## __DESCRIPTION__

This command will plan and build for the given target under the assumption it
is a test.

## __OPTIONS__

`--top <unit>`  
      Set the top level design unit

`--bench <unit>`  
      Set the top level testbench unit

`--fileset <key=glob>...`  
      A glob-style pattern identified by name to include in the blueprint

`--clean`  
      Remove all previous files from the target directory before execution

`--force`  
      Ignore reading the precomputed lock file

`--target <name>`  
      Target to execute

`--all`  
      Include all locally found HDL files

`--command <cmd>`  
      Command to execute

`--list`  
      View available targets

`--target-dir <dir>`  
      The relative directory to locate the blueprint file

`--verbose`  
      Display the command being executed

`args`  
      Arguments to pass to the target

## __EXAMPLES__

```
orbit test --top top --target modelsim -- --lint
```

