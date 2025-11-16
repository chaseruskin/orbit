# __orbit doc__

## __NAME__

doc - generate a project's documentation

## __SYNOPSIS__

```
orbit doc [options]
```

## __DESCRIPTION__

Generates documentation for a project and all of its dependencies based on
inline markdown comments.

The BROWSER environment variable is required to be set when using the `--open`
flag.

## __OPTIONS__

`--open`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Open the docs in a browser after the operation

`--no-deps`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Don't build documentation for dependencies

`--document-private-items`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Document private items

`--target-dir <dir>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Directory for all generated artifacts and intermediate files

## __EXAMPLES__

```
orbit doc
orbit doc --no-deps
```

