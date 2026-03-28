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

Documentation is extracted from comments embedded in the source code. A
comment on the immediate line above a documentable item will be treated as 
that item's documentation, unless a comment is found on the exact line of the
documentable item. Multiple lines of comments can represent a documentable
item as long as there are no lines between the comments that is not a comment.
Comments used for document generation are interpreted as Markdown.

The BROWSER environment variable is required to be set when using the `--open`
flag.

## __OPTIONS__

`--open`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Open the docs in a browser after the operation

`--no-deps`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Don't build documentation for dependencies

`--document-private-units`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Document private units

`--target-dir <dir>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Directory for all generated artifacts and intermediate files

## __EXAMPLES__

```
orbit doc
orbit doc --no-deps
```

