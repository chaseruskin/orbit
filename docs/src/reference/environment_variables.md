# Environment Variables

Orbit's configuration can be customized with the setting of specific environment variables. These variables can be accessed anytime Orbit is executed.

- `ORBIT_HOME` - Location where Orbit stores its data. By default Orbit reads and writes to "$HOME/.orbit" on Unix systems and "%USERPROFILE%/.orbit" on Windows systems.

- `NO_COLOR` - If set, do not print colorized output to the terminal.

- `ORBIT_WIN_LITERAL_CMD` - If set, disables the default behavior of checking for programs ending with ".exe" then ".bat" when a program name without extension is not found on Windows systems.

## Runtime environment variables

Orbit also sets environment variables during runtime such that any subprocesses within Orbit, such as targets, can access necessary information.

- `ORBIT` - The full path to the `orbit` binary performing the build.

- `ORBIT_MANIFEST_DIR` - The full path to the directory that contains the current project's manifest.

- `ORBIT_MANIFEST_FILE` - The full path to the file that is the current project's manifest.

- `ORBIT_PROJECT_DIR` - The full path to the directory that contains the current project's manifest.

- `ORBIT_PROJECT_NAME` - The name of the current project.

- `ORBIT_PROJECT_UUID` - The uuid of the current project.

- `ORBIT_PROJECT_LIBRARY` - The interpretated HDL library of the current project.

- `ORBIT_PROJECT_VERSION` - The version of the current project.

- `ORBIT_PROJECT_CHECKSUM` - The full 64 character hexadecimal string of the SHA256 checksum of the current project.

- `ORBIT_PROJECT_SOURCE` - The source url of the current project. This environment variable is only available during a protocol's execution.

- `ORBIT_TARGET` - The name of the target selected for the latest build process.

- `ORBIT_TOP_NAME` - The top level design's identifier for the latest build process.

- `ORBIT_TOP_FILE` - The full file system path that contains the top level design for the latest build process.

- `ORBIT_TOP_JSON` - The serialized json data for the top level design unit for the latest build process (see [JSON Output](json.md)).

- `ORBIT_TB_NAME` - The testbench's identifier for the latest build process, only if the build process was a test and a testbench was found.

- `ORBIT_TB_FILE` - The full file system path that contains the testbench for the latest build process, only if the build process was a test and a testbench was found.

- `ORBIT_TB_JSON` - The serialized json data for the testbench for the latest build process, only if the build process was a test and a testbench was found (see [JSON Output](json.md)).

- `ORBIT_DUT_NAME` - The design under test's identifier for the latest build process, only if the build process was a test.

- `ORBIT_DUT_FILE` - The full file system path that contains the design under test for the latest build process, only if the build process was a test.

- `ORBIT_DUT_JSON` - The serialized json data for the design under test for the latest build process, only if the build process was a test (see [JSON Output](json.md)).

- `ORBIT_BLUEPRINT` - The file name for the blueprint created from the planning stage of the latest build process, relative to the current build process's output directory.

- `ORBIT_BLUEPRINT_PLAN` - The plan name for the blueprint created from the planning stage of the latest build process. See [Blueprint](./blueprint.md) for all possible plans.

- `ORBIT_TARGET_DIR` - The directory where all generated artifacts from any targets will be stored, relative to the current project's directory. Default is "target".
  
- `ORBIT_OUT_DIR` - The full path to the directory where all generated artifacts for the current target will be stored. This directory is inside the target directory for the current project, and is unique for each selected target.

- `ORBIT_CHANNEL_NAME` - The name of the current channel being used in the publishing process.

- `ORBIT_CHANNEL_DIR` - The full path to the directory that is the root of the current channel.

- `ORBIT_CHANNEL_PROJECT_DIR` - The full path to the directory in the current channel where the current project's manifest will be copied to during the publishing process.

- `ORBIT_PROTOCOL` - The name of the protocol selected for the downloading the current project. This environment variable is only available during a protocol's execution.


## Checking the environment

See [`orbit env`](./../commands/env.md) for checking environment variables on the command-line. Not all environment variables, especially runtime environment variables, may be available.

To review runtime environment variables, open the `.env` file created by Orbit at the root of a target's output directory. All environment variables found in the `.env` file are set by Orbit during the build process before the execution stage.

<!--
Note about environment variables vs. settings file vs. arguments

precedence:
1. config file
2. env vars
3. command-line
-->