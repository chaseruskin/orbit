# __orbit publish__

## __NAME__

publish - post an ip to a channel

## __SYNOPSIS__

```
orbit publish [options]
```

## __DESCRIPTION__

Performs a series of checks on a local ip and then releases it to its specified
channel(s).

There are multiple checks that are performed before an ip can be published. The
ip must have an up to date lockfile with no relative dependencies. The ip's
manifest must also have a value for the source field. Lastly, Orbit must be
able to construct the hdl source code graph without errors.

Posting an ip to a channel involves copying the ip's manifest file to a path 
within the channel known as the index. For every publish of an ip, the index 
corresponds to a unique path within the channel that gets created by Orbit.

By default, it operates a dry run, performing all steps in the process except
for posting the ip to its channel(s). To fully run the command, use the
`--ready` flag. When the ip is published, it will also be installed to the cache
by default. To skip this behavior, use the `--no-install` flag.

## __OPTIONS__

`--ready, -y`  
      Perform a full run

`--list`  
      View available channels and exit

## __EXAMPLES__

```
orbit publish
orbit publish --ready
```

