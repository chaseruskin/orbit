# __orbit publish__

## __NAME__

publish - post a project to a channel

## __SYNOPSIS__

```
orbit publish [options]
```

## __DESCRIPTION__

Performs a series of checks for a local project and then releases it to its
specified channel(s).

There are multiple checks that are performed before a project can be published. 
First, the project must have an up to date lockfile with no relative 
dependencies. The project's manifest must also have a value for the source 
field. In addition, Orbit must be able to construct the HDL source code graph 
without errors. Finally, the project is downloaded from its repository url and 
temporarily installed to verify its contents match those of the local project.

Posting a project to a channel involves copying the project's manifest file to 
a path within the channel known as the index. For every publish of a project, 
the index corresponds to a unique path within the channel that gets created by 
Orbit. A channel's pre-publish and post-publish hooks can get the value for the
project's index by reading the ORBIT_CHANNEL_PROJECT_DIR environment variable.

The `--all-public` flag can be used to skip providing the "project.public" 
field in the current project's manifest, granting the assumption that all 
source files should be public. It cannot be used if the "project.public" field 
exists.

If at least one channel is specified on the command-line using the `--channel`
option, then any channel list found in the project's manifest under the 
"project.channels" field will be ignored as well as any channel list found in the
"publish.default-channels" field of a configuration file.

By default, this command performs a dry run, which executes all of the steps 
in the process except for actually posting the project to its channel(s). 
To run the command to completion, use the `--ready` option.

If the publishing process fails during the channel's pre or post commands,
Orbit will rollback its changes by removing any files it copied into the
channel as well as any newly created directories it made.

## __OPTIONS__

`--ready, -y`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Run the operation to completion

`--channel, -c <name>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Use this channel during publishing

`--no-install`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not install the project for future use

`--list, -l`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; View available channels and exit

`--all-public`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Publish with all source files being public

## __EXAMPLES__

```
orbit publish
orbit publish --ready
orbit publish -c hyperspace-labs -y
```

