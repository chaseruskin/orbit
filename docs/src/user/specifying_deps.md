# Specifying Dependencies

Your projects can depend on other projects found in the catalog or subdirectories on your local file system. You can also temporarily override the location of a dependency - for example, to be able to test out a bug fix in the dependency that you are working on locally. You can also have dependencies that are only used during development. This section walks through the different ways you can specify the dependencies of a project.

### Assumptions

The guides in this section assume you are running commands from the root directory or any subdirectory of the current project. The current project is the project being actively developed.

### Guides
- [Using a strict version of a dependency](#using-a-strict-version-of-a-dependency)
- [Using a relaxed micro version of a dependency](#using-a-relaxed-micro-version-of-a-dependency)
- [Using a relaxed minor version of a dependency](#using-a-relaxed-minor-version-of-a-dependency)
- [Overriding the location of a dependency](#overriding-the-location-of-a-dependency)
- [Including a dependency only for development](#including-a-dependency-only-for-development)
- [Differentiating among dependencies of the same name](#differentiating-among-dependencies-of-the-same-name)

## Using a strict version of a dependency

1. Add the name of the external project (such as `gates`) as a new entry in the `[dependencies]` table of the current project's manifest with the fully qualified version to use (such as `1.0.0`):
``` toml
[dependencies]
gates = "1.0.0"
```

## Using a relaxed micro version of a dependency

1. Add the name of the external project (such as `gates`) as a new entry in the `[dependencies]` table of the current project's manifest with a partially qualified version to use, omitting the micro version number (such as `1.0`) to accept _any_ micro version number:
``` toml
[dependencies]
gates = "1.0"
```

## Using a relaxed minor version of a dependency

1. Add the name of the external project (such as `gates`) as a new entry in the `[dependencies]` table of the current project's manifest with a partially qualified version to use, omitting the minor and micro version number (such as `1`) to accept _any_ minor and _any_ micro version number:
``` toml
[dependencies]
gates = "1"
```

## Overriding the location of a dependency

1. Specify the local file sytem path to the root directory of the external project using the `path` field to use it as the source for the dependency:
``` toml
[dependencies]
gates = { path = "../gates" }
```

### Multiple locations

It is possible to specify both a version and a path location. The path dependency will be used locally (in which case the version is checked against the local copy), and when published to a channel, it will use the version.
``` toml
[dependencies]
gates = { path = "../gates", version = "2.1" }
```
An example where this can be useful is when you have split up a project into multiple projects within the same repository. You can then use path dependencies to point to the local projects within the repository to use the local version during development, and then use the version once it is published.

## Including a dependency only for development

1. Add the name of the external project (such as `gates`) as a new entry in the `[dev-dependencies]` table of the current project's manifest:
``` toml
[dev-dependencies]
gates = "1.0.0"
```

> __Tip__: All methods of specifying dependencies using the `[dependencies]` table can also be applied to the `[dev-dependencies]` table.

## Differentiating among dependencies of the same name

1. Specify the UUID of the external project that has a conflicting name in the catalog using the `uuid` field to explicitly request this project as the dependency:
``` toml
[dependencies]
gates = { uuid = "3p3oajkuoukigs45fskr0svnv", version = "1.0.0" }
```

Note that the `version` field is required.
