# __orbit tree__

## __NAME__

tree - view the dependency graph

## __SYNOPSIS__

```
orbit tree [options]
```

## __DESCRIPTION__

This command will show the hierarchical tree-structure starting from a node.
By default, it will attempt to automatically detect the root if it is
unambiguous and `--root` is not provided. This command only works when called
from the current working ip.

The hdl-level tree displays the connections between entities. The hdl tree does 
not show how many times an entity is instantiated within a parent entity, and 
all architectures for each entity are analyzed. If an unidentified entity is 
instantiated it will appear as a leaf and is denoted as a black box by a '?' 
character.

An entity is considered a black box if it cannot find that referenced entity's 
hdl source code file.

To view the dependency tree at the ip-level, use `--ip`.

## __OPTIONS__

`--root <unit>`  
      The uppermost hdl unit to start the dependency tree

`--compress`  
      Replace duplicate branches with a label marking

`--all`  
      Include all possible roots in the tree

`--format <fmt>`  
      Determine how to display nodes ('long', 'short')

`--ascii`  
      Limit the textual tree characters to the 128 ascii set

`--ip`  
      View the dependency graph at the ip level

## __EXAMPLES__

```
orbit tree --ip
orbit tree --root top --format long
orbit tree --ascii --all
```

