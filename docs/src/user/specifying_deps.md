# Specifying Dependencies

Your ip can depend on other ips found in the catalog or subdirectories on your local file system. You can also temporarily override the location of a dependency - for example, to be able to test out a bug fix in the dependency that you are working on locally. You can also have dependencies that are only used during development. This section walks through the different ways you can specify the dependencies of an ip.

### Assumptions

The guides in this section assume you are running commands from the root directory or any subdirectory of the current ip. The current ip, also sometimes referred to as the local ip, is the ip being actively developed.

### Guides
- [Using a strict version of a dependency](#using-a-strict-version-of-a-dependency)
- [Using a relaxed micro version of a dependency](#using-a-relaxed-micro-version-of-a-dependency)
- [Using a relaxed minor version of a dependency](#using-a-relaxed-minor-version-of-a-dependency)
- [Overriding the location of a dependency](#overriding-the-location-of-a-dependency)
- [Including a dependency only for development](#including-a-dependency-only-for-development)
- [Differentiating among dependencies of the same name](#differentiating-among-dependencies-of-the-same-name)

## Using a strict version of a dependency

1. Add the name of the ip (such as `gates`) as a new entry in the `[dependencies]` table of the current ip's manifest with the fully qualified version to use (such as `1.0.0`):
``` toml
[dependencies]
gates = "1.0.0"
```

## Using a relaxed micro version of a dependency

1. Add the name of the ip (such as `gates`) as a new entry in the `[dependencies]` table of the current ip's manifest with a partially qualified version to use, omitting the micro version number (such as `1.0`) to accept _any_ micro version number:
``` toml
[dependencies]
gates = "1.0"
```

## Using a relaxed minor version of a dependency

1. Add the name of the ip (such as `gates`) as a new entry in the `[dependencies]` table of the current ip's manifest with a partially qualified version to use, omitting the minor and micro version number (such as `1`) to accept _any_ minor and _any_ micro version number:
``` toml
[dependencies]
gates = "1"
```

## Overriding the location of a dependency

1. Specify the local file sytem path to the root directory of the ip using the `path` field to use it as the source for the dependency:
``` toml
[dependencies]
gates = { path = "../gates", version = "1.0.1-dev" }
```

Note that the `version` field is required and its value must match the `version` defined in the local dependency's manifest file.

## Including a dependency only for development

1. Add the name of the ip (such as `gates`) as a new entry in the `[dev-dependencies]` table of the current ip's manifest:
``` toml
[dev-dependencies]
gates = "1.0.0"
```

> __Tip__: All methods of specifying dependencies found in the `[dependencies]` table can also be applied to the `[dev-dependencies]` table.

## Differentiating among dependencies of the same name

1. Specify the UUID of the ip that has a conflicting name in the catalog using the `uuid` field to explicitly request this ip as the dependency:
``` toml
[dependencies]
gates = { uuid = "3p3oajkuoukigs45fskr0svnv", version = "1.0.0" }
```

Note that the `version` field is required.
