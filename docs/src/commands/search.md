# __orbit search__

## __NAME__

search - browse the catalog

## __SYNOPSIS__

```
orbit search [options] [<project>]
```

## __DESCRIPTION__

Returns a list of the projects found in the catalog.

The returned list of projects is organized by row with each row consisting of four
columns: the project's name, latest version, catalog state, and UUID.

By default, all projects in the catalog will be returned. To filter by project
name, use the `<project>` option. To limit the number of results, use the 
`--limit` option.

A project can be stored across three different levels: installed in the cache,
downloaded to the archive, and available via channels. By default, all levels
are searched for projects. Applying a level filter (`--install`, `--download`, 
`--available` options) will restrict the search to only checking the filtered
levels for projects.

A resulting project is only read from one level, even when multiple levels are
searched. When a project exists at multiple levels, the catalog imposes a 
priority on which level to choose. Installed projects have higher priority 
over downloaded projects, and downloaded projects have higher priority over 
available projects.

Results can also be filtered by keyword using the `--keyword` option. By
default, if a project matches at least one filter then it will be returned in 
the result. To collect only projects that match each presented filter, use 
the `--match` option.

If a project has a higher version that exists and is not currently installed, 
then an asterisk character "*" will appear next the project's version. To 
update the project to the latest version, see the `install` command.

## __OPTIONS__

`<project>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Project id specification

`--install, -i`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter projects installed to the cache

`--download, -d`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter projects downloaded to the archive

`--available, -a`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Filter projects available via channels

`--keyword <term>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Include projects that have this keyword

`--limit <n>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Maximum number of results to return

`--match`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Return results that pass each filter

## __EXAMPLES__

```
orbit search axi
orbit search --keyword memory --keyword ecc
orbit search --keyword cdc --limit 20 -i
```

