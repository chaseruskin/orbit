# __orbit tree__

## __NAME__

tree - show a dependency graph

## __SYNOPSIS__

```
orbit tree [options] [<unit>...]
```

## __DESCRIPTION__

Shows the hierarchical tree structure of the hardware design starting from a
root node.

By default, it will try to automatically detect the root node for the 
local project. If there is ambiguity in determining what node can be the root, 
then all root nodes and their respective trees will be displayed. To only 
display the tree of a particular node, use the `<unit>` option.

The tree can display different kinds of dependencies relative to the current
project using the `--edges` option. By default, this command uses "unit". By
specifying edges as "project", it will return the project-level dependency 
tree. When using "unit" or "all", the HDL dependency graph will be displayed.

The HDL graph shown with "unit" displays the composition of usable 
entities/modules. To generate this graph, it analyzes each VHDL architecture 
and ignores Verilog compiler directives. If an unidentified entity is 
instantiated, it will appear as a leaf in the graph and will be considered 
as a "black box" denoted by the "?" character next to its position in the 
tree. The HDL graph shown with "all" displays the composition of the design 
including all primary design unit references. Any references (excluding entity 
instantiations) that are not found will not appear in the dependency graph for
the "all" option.

Nodes marked with (*) have been "de-duplicated". The dependencies for the node 
have already been shown elsewhere in the graph, and so are not repeated. Use 
the `--no-dedupe` option to repeat the duplicates.

Using the `--format` option can alter how much information is displayed for
each HDL design unit in the tree composition. By default, only the design
unit's name is displayed for each unit.

If the tree's character output is not displaying properly, then the tree can
be displayed using a set of standard ASCII characters with the `--charset`
option set to "ascii".

## __OPTIONS__

`<unit>...`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Uppermost hdl unit of the dependency tree

`--edges, -e <kind>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; The kind of dependencies to display (unit, project, all)

`--invert, -i`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Show the reverse dependencies for the node

`--depth <depth>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Maximum display depth of the dependency tree

`--format <format>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Determine how to display node names (short, long)

`--no-dedupe`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Do not de-duplicate repeated dependencies

`--charset <charset>`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Choose the character set for the tree (utf8, ascii)

`--json`  
&nbsp; &nbsp; &nbsp; &nbsp; &nbsp; &nbsp; Export the tree's information as valid json

## __EXAMPLES__

```
orbit tree
orbit tree top --format long
orbit tree -e project --charset ascii
```

