# __orbit info__

## __NAME__

info - display information about a project

## __SYNOPSIS__

```
orbit info [options] [<project>]
```

## __DESCRIPTION__

Displays various bits of information about a particular project. If no project
is provided, then it displays information related to the local project.

To display manifest information, no additional options are required.

To display the available units within the project, use the `--units` option. 
For non-local projects, its protected and private design units are hidden
from the results. To include design units of all visibility levels, use the 
`--force` flag.

To display the known versions for a project, use the `--versions` option.

## __OPTIONS__

`<project>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Project id specification

`--versions, -v`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the list of known versions

`--units, -u`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Display the units for this project

## __EXAMPLES__

```
orbit info --units
orbit info gates:1.0.0 --units --force
orbit info gates --versions
orbit info gates:1 --versions
```

