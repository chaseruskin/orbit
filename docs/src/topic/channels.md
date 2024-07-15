# Channels

As your codebase evolves over time, you may have accrued a lot of ips. However, an issue arises regarding _discovery_- how do others quickly find all ips that have been released?

Orbit solves this problem by using channels. A _channel_ is a decentralized registry that agglomerates ip metadata into a single location. With this approach, users only need to specify channels to discover the wide variety of released ips.

Channels can be as manual or automated as you prefer. You can configure commands to run for a channel's synchronization sequence, pre-launch sequence, and post-launch sequence. Channels are encouraged to be as automated as possible by defining these fields the channel's configuration.

## Adding a new ip to a channel

Orbit automates the process of adding an ip to a channel with `orbit launch`.

The ip's manifest data gets placed in the channel by using the following pattern:

```
{{orbit.ip.char}}/{{orbit.ip.name}}-{{orbit.ip.version}}-{{orbit.ip.uuid}}.toml
```

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

# Enable Orbit to discover ips from this channel
allow-read = true
# Enable Orbit to add new ips to this channel
allow-write = true

```