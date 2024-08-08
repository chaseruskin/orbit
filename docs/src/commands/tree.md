# __orbit tree__

## __NAME__

tree - show the dependency graph

## __SYNOPSIS__

```
orbit tree [options]
```

## __DESCRIPTION__

Shows the hierarchical tree structure of the hardware design starting from a
root node.

By default, it will try to automatically detect the root node for the 
local ip. If there is ambiguity in determining what node can be the root, then 
all root nodes and their respective trees will be displayed. To only display
the tree of a particular node, use the `--root` option.

There are two trees available to view: hdl and ip. By default, the hdl
dependency graph is displayed. The hdl graph shows the composition of usable 
entities/modules. To generate this graph, it analyzes each VHDL architecture 
and ignores Verilog compiler directives. If an unidentified entity is 
instantiated, it will appear as a leaf in the graph and will be considered as 
a "black box" denoted by the "?" character next to its position in the tree.

Using the `--format` option can alter how much information is displayed for
each hdl design unit in the tree composition. By default, only the design
unit's name is displayed for each unit.

To display the ip dependency graph, use the `--ip` option.

If the tree's character output is not displaying properly, then the tree can
be displayed using a set of standard ASCII characters with the `--ascii`
option.

## __OPTIONS__

`--root <unit>`  
      The uppermost hdl unit of the dependency tree

`--format <fmt>`  
      Determine how to display nodes ('long', 'short')

`--ascii`  
      Limit the textual tree characters to the 128 ascii set

`--ip`  
      Switch to the ip dependency graph

## __EXAMPLES__

```
orbit tree
orbit tree --root top --format long
orbit tree --ip --ascii
```

