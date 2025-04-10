# Releasing Ip

After the ip has been tested and developed to its specification, it may be ready to give its implementation a meaningful version and release it for others to use. This section walks through the common steps to successfully prepare and deploy an ip for use in future ips.

### Assumptions

The guides in this section assume you are running commands from the root directory or any subdirectory of the current ip. The current ip, also sometimes referred to as the local ip, is the ip being actively developed.

This section also assumes you have one or more channels already configured. To learn how to create channels, see [Creating Channels](./creating_channels.md).

Although not strictly necessary, you may also need to have a protocol configured that meets your requirements for how you access an ip from the internet. To learn how to create protocols, see [Creating Protocols](./creating_protocols.md).

### Guides

- [Publishing an ip using git tags](#publishing-an-ip-using-git-tags)

## Publishing an ip using git tags

This guide walks through the steps to have a successful release using a channel and git tags. It assumes your ip is a git repository and has a remote repository on a platform such as GitHub.

1. Specify what version is going to be released:
``` toml
[ip]
# ...
version = "1.0.0"
```

2. Define a source url for the project:
``` toml
[ip]
# ...
source = "https://github.com/chaseruskin/gates/archive/{{orbit.ip.version}}.zip"
```

3. Specify what source files are public:
``` toml
[ip]
# ...
public = ["rtl"]
```

4. Specify which channel to post this to ip to:
``` toml
[ip]
# ...
channels = ["remote-chipyard"]
```

5. Update the lockfile:
```
$ orbit lock
```

6. Add and commit the final changes to your project:
```
$ git commit -am "Stages for release"
```

7. Push the committed changes to the remote repository:
```
$ git push
```

8. Add a git tag to the ip's repository that matches exactly the version specified in the ip's manifest:
```
$ git tag 1.0.0
```

9. Push the tag to the remote repository:
```
$ git push --tags
```

10. Run the dry run for the publishing process to verify all ip checkpoints are met:
```
$ orbit publish
```

11.  Once the dry run reports the ip is ready to be published, publish the ip:
```
$ orbit publish --ready
```