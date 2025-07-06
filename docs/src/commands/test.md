# __orbit test__

## __NAME__

test - run a test

## __SYNOPSIS__

```
orbit test [options] [--] [args]...
```

## __DESCRIPTION__

This command prepares a given target and then executes the target under the
context of a test.

While this command functions similar to `orbit build`, the targets that are 
encouraged to be used with this command are ones that are designed to either
"pass" or "fail", typically through a return code. This command optionally
requests a testbench, if you must not allow a testbench, see `orbit build`.

A target must be provided for the test command to run. A default target can
be specified in a configuration file, which will be used when a target is
omitted from the command-line.

If `--list` is used, then it will display a list of the available targets to
the user. Using `--list` in combination with a target from `--target` will
display any detailed help information the target has documented in its 
definition.

A target typically goes through three steps for the testing process:  
   1. Parse the blueprint file  
   2. Process the referenced files listed in the blueprint  
   3. Verify the HDL source code passes all tests

Any command-line arguments entered after the terminating flag `--` will be
passed in the received order as arguments to the subprocess's command. If a 
target already has defined arguments, the additional arguments passed from the 
command-line will follow the previously defined arguments.

The target's process will spawn from the current project's output directory, 
which is $ORBIT_OUT_DIR.

While the execution stage is a subprocess within Orbit, any non-zero error code 
returned from the user-defined execution process is propagated through Orbit.

## __OPTIONS__

`--target, -t <name>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Target to execute

`--dut <unit>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the device under test

`--tb <unit>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the top level testbench unit

`--plan <format>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the blueprint file format

`--target-dir <dir>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; The relative directory where the target starts

`--command <path>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Overwrite the target's command

`--list, -l`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; View available targets and exit

`--all`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Include all hdl files of the current project

`--fileset <key=glob>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; A glob-style pattern identified by name to include in the blueprint

`--keep, -k`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not clean the target folder before execution

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Force the target to execute 

`--verbose`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the command being executed

`args`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Arguments to pass to the target

## __EXAMPLES__

```
orbit test --dut adder --tb adder_tb --target modelsim -- -r sim
```

