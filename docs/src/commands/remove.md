# __orbit remove__

## __NAME__

remove - delete a project from the catalog

## __SYNOPSIS__

```
orbit remove [options] <project>
```

## __DESCRIPTION__

Deletes save data for a known project from the catalog. The project's data for 
its particular version is removed from the catalog's cache and the catalog's
archive.

By default, an interactive prompt will appear to confirm with the user if the 
correct project is okay to be removed. To skip this interactive prompt and assume
it is correct without confirmation, use the `--force` option.

To add a project to the catalog, see the `install` command.

## __OPTIONS__

`<project>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Project id specification

`--force`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Skip interactive prompts

`--verbose`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display where the removal occurs

## __EXAMPLES__

```
orbit remove gates
orbit remove gates:1.0.1 --force
```

