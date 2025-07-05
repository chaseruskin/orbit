# Releasing Projects

After the current project has been tested and developed to its specification, it may be ready to give its implementation a meaningful version and release it for others to use. This section walks through the common steps to successfully prepare and deploy a project for use in future projects.

### Assumptions

The guides in this section assume you are running commands from the root directory or any subdirectory of the current project. The current project is the local project being actively developed.

This section also assumes you have one or more channels already configured. To learn how to create channels, see [Creating Channels](./creating_channels.md).

Although not strictly necessary, you may also need to have a protocol configured that meets your requirements for how you access a project from the internet. To learn how to create protocols, see [Creating Protocols](./creating_protocols.md).

### Guides

- [Installing a project to the catalog](#installing-a-project-to-the-catalog)
- [Publishing a project using git tags](#publishing-a-project-using-git-tags)

## Installing a project to the catalog

This guide walks through the steps to install a project from the local file system to the catalog. In this example, we assume the current project has no revision control tied to its project.

1. Specify what version is going to be released using the project manifest's `version` field, such as `1.0.0`:
``` toml
[project]
# ...
version = "1.0.0"
```

1. Specify what source files in the project are public using the project manifest's `public` field:
``` toml
[project]
# ...
public = ["rtl"]
```

1. Update the lockfile:
```
$ orbit lock
```

1. Install the project to the catalog as an immutable reference for use in future projects:
```
$ orbit install --path .
```

## Publishing a project using git tags

This guide walks through the steps to have a successful release using a channel and git tags. It assumes your current project is a git repository and has a remote repository on a platform such as GitHub.

1. Specify what version is going to be released using the project manifest's `version` field, such as `1.0.0`:
``` toml
[project]
# ...
version = "1.0.0"
```

2. Define a source url for the project:
``` toml
[project]
# ...
source = "https://github.com/chaseruskin/gates/archive/{{orbit.project.version}}.zip"
```

3. Specify what source files in the project are public using the project manifest's `public` field:
``` toml
[project]
# ...
public = ["rtl"]
```

4. Update the lockfile:
```
$ orbit lock
```

5. Add and commit the final changes to your project:
```
$ git commit -am "Stages for release"
```

6. Add a git tag to the project's repository that matches exactly the version specified in the project's manifest:
```
$ git tag 1.0.0
```

7. Push the committed changes and tag to the remote repository:
```
$ git push --tags
```

8.  Run the dry run for the publishing process to verify all project checkpoints are met when publishing to an existing channel, such as `remote-chipyard`:
```
$ orbit publish --channel remote-chipyard
```

9. Once the dry run reports the project is ready to be published, publish the project to an existing channel, such as `remote-chipyard`:
```
$ orbit publish --channel remote-chipyard --ready
```