# __orbit analyze__

## __NAME__

analyze - print a blueprint

## __SYNOPSIS__

```
orbit analyze [options]
```

## __DESCRIPTION__

Prints the blueprint to stdout.

This command performs the planning stage of the build process and displays
the resulting blueprint contents to stdout. Entries in the blueprint are
guaranteed to be in a deterministic order.

If the `--plan` option is omitted, then the default plan is used (tsv).

If `--local` is used, then only the entries for source files belonging to
the current project are included in the final blueprint. If the blueprint
includes dependency information from a different project, the dependency 
information is still included for that entry.

If `--force` is used, then a blueprint will be displayed (albeit possibly
incomplete) regardless if there are HDL parsing or analysis errors.

## __OPTIONS__

`--local`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter entries that belong to the working project

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Force the blueprint to be displayed

`--plan <format>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Set the blueprint file format

## __EXAMPLES__

```
orbit analyze --plan json --local
```

