# __orbit publish__

## __NAME__

publish - post an ip to a channel

## __SYNOPSIS__

```
orbit publish [options]
```

## __DESCRIPTION__

Performs a series of checks for a local ip and then releases it to its
specified channel(s).

There are multiple checks that are performed before an ip can be published. 
First, the ip must have an up to date lockfile with no relative dependencies. 
The ip's manifest must also have a value for the source field. In addition,
Orbit must be able to construct the hdl source code graph without errors.
Finally, the ip is downloaded from its source url and temporarily installed
to verify its contents match those of the local ip.

Posting an ip to a channel involves copying the ip's manifest file to a path 
within the channel known as the index. For every publish of an ip, the index 
corresponds to a unique path within the channel that gets created by Orbit.
A channel's pre-publish and post-publish hooks can get the value for the ip's 
index by reading the ORBIT_CHANNEL_IP_DIR environment variable.

The `--all-public` flag can be used to skip providing the "ip.public" field
in the current ip's manifest, granting the assumption that all source files
should be public. It cannot be used if the "ip.public" field exists.

By default, this command performs a dry run, which executes all of the steps 
in the process except for actually posting the ip to its channel(s). 
To run the command to completion, use the `--ready` option.

If the publishing process fails during the channel's pre or post commands,
Orbit will rollback its changes by removing any files it copied into the
channel as well as any newly created directories it made.

## __OPTIONS__

`--ready, -y`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Run the operation to completion

`--no-install`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not install the ip for future use

`--list`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; View available channels and exit

`--all-public`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Publish with all source files being public

## __EXAMPLES__

```
orbit publish
orbit publish --ready
```

