# Upgrading

Once Orbit is installed, it can be self-upgraded to the latest official version released found on its Github.

```
$ orbit --upgrade
```

This behavior performs the following strategy:

1. Connect to https://github.com/c-rus/orbit/releases to find the most recent released version.

2. Check if the most recent version online is ahead of the currently installed version. 

> __Note__: If the version online is ahead, it will prompt you to confirm you wish to install to the new version. This will be able to be skipped with a `--force` flag.

3. Tries to find a suitable package for the current operating system and downloads it along with the checksum file.

4. Computes the checksum for the downloaded package and verifies the checksum matches with the one listed in the file.

5. Removes any executable in the executable's directory starting with `orbit-`, and renames the current executable to `orbit-<version>`.

6. Decompresses the downloaded executable and places it in the current executable's directory.