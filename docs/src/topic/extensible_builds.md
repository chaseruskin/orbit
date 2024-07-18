# Extensible Builds

Orbit is an extensible build tool for HDLs. Orbit separates the build process into two stages: planning and execution. When the build process happens, both stages are operated together in sequential order. Orbit provides two entry points into the build process: `orbit test` and `orbit build`.

What makes Orbit extensible? Well, Orbit does not define the execution stage by default. It leaves it upon the user to add their own execution processes, called _targets_. A target can be added through modifying an Orbit configuration file.

Orbit leaves the execution stage undefined by default because there are a wide range of different backend EDA tools available that enforce different requirements and even change requirements and behaviors across versions. It would be a nightmare to try to design a "one-script-fits-all" approach because everyone's computing system and choice of tool is so diverse.

## Test or build?

Orbit provides two entry points into the build process: `orbit test` and `orbit build`. Each entry point is suited for a particular type of build process.

If you are trying to run a simulation (accompanied by an HDL testbench), then you should use the `orbit test` entry. This command allows you to enter the build process by specifying the testbench using `--tb <unit>` and its design-under-test using `--dut <unit>`. This entry is typically used for verification workflows, where the end result of the build process is more concerned about making sure all steps in the process complete successfully with no errors.

For any non-testing workflow (one that lacks an HDL testbench), then you should use the `orbit build` entry. This command allows you to enter the build process by specifying the top level using `--top <unit>`. This entry is typically used for any workflow where the end result of the build process is more concerned about producing output files (commonly called artifacts), such as a bitstream or synthesis report.

## Planning

During the planning stage, Orbit resolves all source code dependencies to generate a single file that lists all the necessary source files in topologically sorted order. This file that stores the ordered list of source file paths is called the _blueprint_. 

Orbit sets runtime environment variables that can be accessed during the execution stage by the specified target.

## Execution

The execution stage occurs after the planning stage. During the execution stage, Orbit invokes the specified target's command with its set of determined arguments. The arguments to include are taken from the predefined list in the configuration file as well as any additional arguments found on the command line that appear after an empty double switch (`--`).

Typically, the target's process involves reading the blueprint previously generated from the planning stage and performing some task to generate an _artifact_. An artifact is what the build produces at the end of its execution, which may be anything, from a synthesis report to a bitstream file.