# Channels

As your codebase evolves over time, you may have accrued a lot of ips. However, an issue arises regarding _discovery_- how do others quickly find all ips that have been released?

Orbit solves this problem by using channels. A _channel_ is a lightweight decentralized registry index. In other words, a channel is a directory that contains multiple ip manifests. With this approach, users can simply configure channels to discover the many available released ips.

Channels can be as manual or automated as you prefer. You can configure commands to run for a channel's synchronization hook, pre-publish hook, and post-publish hook. Channels are encouraged to be as automated as possible by defining these fields in the channel's configuration.

### Example

``` toml
[[channel]]
name = "hyperspace-labs"
description = "Available ip from hyperspace labs"
# Directory where ip contents are placed
root = "./index"

# If the channel is stored on the internet, synchronize with its remote location
sync.command = "git"
sync.args = ["pull"]

# Issue this command immediately before adding the ip to the channel
pre.command = "git"
pre.args = ["pull"]

# Issue this command immediately after adding the ip to the channel
post.command = "python"
post.args = ["publish.py"]
```

## The synchronization hook

A synchronization hook can be configured for a channel to update your local copy of a channel with its remote source. Synchronization hooks, if configured, will run as part of the publishing process to confirm the ip has not been released yet.

All channels with synchronization hooks can be manually triggered to run their synchronization hooks by running:
```
$ orbit --sync
```

## The pre-publish hook

A pre-publish hook can be configured for a channel to perform any operations immediately before the ip's metadata contents are copied into the given channel.

## The post-publish hook

A post-publish hook can be configured for a channel to perform any operations immediately after the ip's metadata contents are copied into the given channel.

## The publishing process

Orbit automates the process of releasing an ip to a channel through the publishing process invoked through `orbit publish`.

The core operation during the publishing process is the ip's manifest gets placed in the channel at its generated index directory. The root of the index directory can be configured using the `root` field, and the remaining subdirectories are pre-determined according to the ip's name. 

The full path to the index directory can be read from the `ORBIT_CHANNEL_DIR` environment variable during a channel's pre-publish or post-publish hook processes. 

The full path to the directoy when an ip's metadata contents are copied to during the publishing process can be read from the `ORBIT_CHANNEL_IP_DIR` environment variable.
