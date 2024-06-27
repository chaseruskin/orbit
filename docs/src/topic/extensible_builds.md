# Extensible Builds

Orbit is an extensible build tool for HDLs. Orbit separates the build process into two stages: planning and execution. Both stages are operated together when `orbit build` is called.

Users can add their own execution processes, called _targets_. By default, Orbit does not have any built-in targets. A target can be added by editing the configuration file.

## Planning

During the planning stage, Orbit resolves all source code dependencies to generate a single file that lists all the necessary source files in topologically sorted order. This file that stores the ordered list of source file paths is called the _blueprint_. 

Orbit sets runtime environment variables that can be accessed during the execution stage by the specified target.

## Execution

The execution stage occurs after the planning stage. During the execution stage, Orbit invokes the specified target's command with its set of determined arguments. The arguments to include are taken from the predefined list in the configuration file as well as any additional arguments found on the command line that appear after an empty double switch (`--`).

Typically, the target's process involves reading the blueprint previously generated from the planning stage and performing some task to generate an _artifact_. An artifact is what the build produces at the end of its execution, which may be anything, from a synthesis report to a bitstream file.