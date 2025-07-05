# Creating Channels

A channel is a decentralized registry storing the manifests of published projects. This section provides steps for ways to configure a channel.

### Guides
- [Configuring a local directory as a channel](#configuring-a-local-directory-as-a-channel)
- [Configuring a git repository as a channel](#configuring-a-git-repository-as-a-channel)
- [Configuring a channel as default](#configuring-a-channel-as-default)
- [Viewing available channels](#viewing-available-channels)

## Configuring a local directory as a channel

This guide outlines how to set up a directory on the local file system as a recognized channel. In this guide, our channel is called `local-chipyard`.

1. Create a new directory in the Orbit home directory:
```
mkdir -p "$(orbit env ORBIT_HOME)/channels/local-chipyard"
```

2. Open the global Orbit configuration file located at `$ORBIT_HOME/config.toml`.

3. Make a new entry in the `[[channel]]` array:
``` toml
[[channel]]
name = "local-chipyard"
description = "Local registry of available projects"
root = "./channels/local-chipyard"
```

4. View your available channel:
```
orbit publish --list
```

## Configuring a git repository as a channel

This guide outlines one way to create a channel that can be automatically tracked using git. In this guide, our channel is called `remote-chipyard`.

1. Create a new repository along with a remote configured for it to allow pushing and pulling.
   
2. Add an Orbit configuration file at the root of the repository called `config.toml`:
```
$ touch config.toml
```

3. Add a new entry in the `[[channel]]` array for the configuration file:
``` toml
[[channel]]
name = "remote-chipyard"
description = "Remote registry of available projects"
```

4. Specify the directory where data for published projects should exist within the repository:
``` toml
[[channel]]
# ...
root = "./index"
```

5. Set up synchronization hooks for this channel such that users that have this channel configured can seamlessly get updates using `orbit --sync`:
``` toml
[[channel]]
# ...
sync.command = ["git", "pull"]
```

1. Create a Python script called `publish.py` to handle automatically adding, committing, and pushing published projects:
``` Python
import subprocess
import os

project_index = os.environ.get("ORBIT_CHANNEL_PROJECT_DIR")
project_name = os.environ.get("ORBIT_PROJECT_NAME")
project_version = os.environ.get("ORBIT_PROJECT_VERSION") 

# Add untracked files
child = subprocess.Popen(['git', 'add', project_index])
rc = child.wait()
if rc != 0:
    exit(rc)
# Commit the changes
child = subprocess.Popen(['git', 'commit', '-m', "Publishes "+project_name+':'+project_version])
rc = child.wait()
if rc != 0:
    exit(rc)
# Push the changes to the remote
child = subprocess.Popen(['git', 'push'])
rc = child.wait()
if rc != 0:
    exit(rc)
```

7. Configure the `publish.py` Python script to run within the publishing process after Orbit copies the published project metadata into the channel:
``` toml
[[channel]]
# ...
post.command = ["python", "publish.py"]
```

8. Add, commit, and push these changes to the git repository.

9. Modify the global Orbit configuration file to include this repository's configuration file:
```
$ orbit config --push include="$PWD/config.toml"
```

10. View the configured channel:
```
$ orbit publish --list
```

## Configuring a channel as default

This guide walks through how to configure existing channels to be the default when a project does not specify any particular channels during the publishing process.

1. Open an Orbit configuration file.

2. Add the `default-channels` field in the `[publish]` section, where the value is an array of one or more strings representing the names of a previously defined channels (such as `local-chipyard` and `remote-chipyard`):
``` toml
[publish]
default-channels = ["local-chipyard", "remote-chipyard"]
```

## Viewing available channels

This guide walks through how to view the channels already configured and available for a project to publish to.

1. Return the list of configured channels:
``` 
$ orbit publish --list
```

2. Return configuration data about a particular channel, such as `local-chipyard`:
```
$ orbit publish --list --channel local-chipyard
```