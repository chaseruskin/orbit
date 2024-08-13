# Channels

As your codebase evolves over time, you may have accrued a lot of ips. However, an issue arises regarding _discovery_- how do others quickly find all ips that have been released?

Orbit solves this problem by using channels. A _channel_ is a lightweight decentralized registry index. In other words, a channel is a directory that contains multiple ip manifests. With this approach, users can simply configure channels to discover the many available released ips.

Channels can be as manual or automated as you prefer. You can configure commands to run for a channel's synchronization hook, pre-publish hook, and post-publish hook. Channels are encouraged to be as automated as possible by defining these fields in the channel's configuration.

## Adding a new ip to a channel

Orbit automates the process of adding an ip to a channel with `orbit publish`.

The ip's manifest gets placed in the channel by using its generated index path. The index path can be read from the `ORBIT_CHAN_INDEX` environment variable during a channel's pre-publish or post-publish hook processes.

## Example

``` toml
[[channel]]
name = "hyperspace-labs"
description = "Available ip from hyperspace labs"
root = "." # Optional, default is "."

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