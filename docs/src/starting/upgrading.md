# Upgrading

Once Orbit is installed, it can be self-upgraded to the latest official released version found on its Github repository.
```
$ orbit --upgrade
```

This behavior performs the following strategy:

1. Removes any executable in the executable's directory starting with `orbit-` (these are considered stale binaries, such as `orbit-0.1.0`).

2. Connects to [https://github.com/c-rus/orbit/releases](https://github.com/c-rus/orbit/releases) to find the most recent released version.

3. Checks if the most recent version online is ahead of the currently installed version.

> __Note__: If the version online is newer, a prompt will appear to confirm you wish to install the new version. This prompt can be bypassed by adding the `--force` flag to the previous command.

4. Downloads the checksum file to a temporary directory to see if there is a prebuilt package available for the current architecture and operating system.

5. Downloads the package to a temporary directory and computes the checksum to verify the contents.

6. Renames the current executable by appending its version to the name (marking it as a stale binary, such as `orbit-0.1.1`).

7. Unzips the package and moves the new executable to the original executable's location.


> __Note__: If you wish to remove the newly created stale binary after an upgrade, rerunning `orbit --upgrade` immediately again will perform step 1 and stop at step 3.