# Extensible Builds

Orbit is an extensible build system for HDLs. Orbit separates the build process into two stages: planning and execution. When the build process happens, both stages are performed together in sequential order. Orbit provides two entry points into the build process: `orbit test` and `orbit build`.

What makes Orbit extensible? Well, Orbit does not define the execution stage by default. It leaves it upon the user to add their own execution processes, called _targets_. A target can be added by modifying an Orbit configuration file. You can write target scripts for whatever tools are integral to your workflow, such as Vivado, ModelSim, VCS, Design Compiler, or Quartus Prime.

![](./../images/dataflow.svg)

## Planning stage

Orbit is the central system of the planning stage. Orbit takes your HDL source files (VHDL, Verilog, and SystemVerilog files) as input and generates the topologically sorted order of the source files saved to a single file known as the _blueprint_ as output. Alongside the blueprint, Orbit sets a series of environment variables specific to the current build process that are available to the specified target during the execution stage for that target's convenience. These environment variables include things such as the output directory (`ORBIT_OUT_DIR`) and the name of the top-level design unit (`ORBIT_TOP_NAME`).

To create the blueprint, Orbit tokenizes your current project's HDL source code along with the HDL source code of the current project's depedendencies found in the catalog to analyze references to other design units across source files. With this information, Orbit generates a graph at the HDL source code level and sorts it in topological order, with the top-level design unit being last.

<!-- Orbit sets runtime environment variables that can be accessed during the execution stage by the specified target. -->

## Execution stage

The target specified at the start of the build process is the central system of the execution stage. The target typically takes in the outputs from the planning stage (blueprint and environment variables) as inputs to perform some backend processing with an EDA tool. Finally, the target executes the backend EDA tool to produce your results, also known as _artifacts_. Artifacts can really be anything, from synthesis reports and log files to netlists and bitstreams. Recall that targets are written and configured by _you_, so their behavior is in your control. 

Orbit leaves the execution stage undefined by default because there are a wide range of different backend EDA tools available that enforce different requirements and even change requirements and behaviors across versions. It would be a nightmare to try to design a "one-script-fits-all" approach because everyone's desired workflow, computing requirements, and choice of tool is so diverse.

It is encouraged for a target to read the environment variables set by Orbit during the planning stage to further tailor the executed workflow for the given set of inputs. The environment variables can be used to help write robust and resuable targets.

## Test or build?

Orbit provides two entry points into the build process: `orbit test` and `orbit build`. Each entry point is suited for a particular type of build process.

If you are trying to run a simulation (accompanied by an HDL testbench), then it is recommended to use the `orbit test` entry point. This command allows you to enter the build process by specifying the testbench using `--tb <unit>` and its design-under-test using `--dut <unit>`. This entry point is typically used for verification workflows, where the end result of the build process is more concerned with making sure all steps in the process complete successfully with no errors.

For any non-testing workflow (one that lacks an HDL testbench), then it is recommended to use the `orbit build` entry point. This command allows you to enter the build process by specifying the top level using `--top <unit>`. This entry point is typically used for any workflow where the end result of the build process is more concerned with producing output files (commonly called artifacts), such as a bitstream or synthesis report.