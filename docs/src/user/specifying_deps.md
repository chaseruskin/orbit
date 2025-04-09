# Specifying Dependencies

Your ip can depend on other ips found in the catalog or subdirectories on your local file system. You can also temporarily override the location of a dependency - for example, to be able to test out a bug fix in the dependency that you are working on locally. You can also have dependencies that are only used during development. This section walks through the different ways you can specify the dependencies of an ip.

### Assumptions

The guides in this section assume you are running commands from the root directory or any subdirectory of the current ip. The current ip, also sometimes referred to as the local ip, is the ip being actively developed.

### Guides
- [Using a strict version of a dependency](#using-a-strict-version-of-a-dependency)
- [Using a relaxed micro version of a dependency](#using-a-relaxed-micro-version-of-a-dependency)
- [Using a relaxed minor version of a dependency](#using-a-relaxed-minor-version-of-a-dependency)
- [Overriding the location of a dependency](#overriding-the-location-of-a-dependency)
- [Differentiating among dependencies of the same name](#differentiating-among-dependencies-of-the-same-name)

## Using a strict version of a dependency

``` toml
[dependencies]
gates = "1.0.0"
```

## Using a relaxed micro version of a dependency

``` toml
[dependencies]
gates = "1.0"
```

## Using a relaxed minor version of a dependency

``` toml
[dependencies]
gates = "1"
```

## Overriding the location of a dependency

``` toml
[dependencies]
gates = { path = "../gates", version = "1.0.1-dev" }
```

## Differentiating among dependencies of the same name

``` toml
[dependencies]
gates = { uuid = "3p3oajkuoukigs45fskr0svnv", version = "1.0.0" }
```
