# __orbit tree__

## __NAME__

tree - view the dependency graph

## __SYNOPSIS__

```
orbit tree [options]
```

## __DESCRIPTION__

This command will show the hierarchical tree-structure starting from a node.
By default, Orbit will attempt to automatically detect the root if it is
unambiguous.

## __OPTIONS__

`--root <entity>`  
      Top-level entity identifier to mark as the root node
 
`--compress`  
      Replace duplicate branches with a label marking
 
`--all`  
      Include all possible roots in hierarchical tree
 
`--format <fmt>`  
      Select how to display entity names: 'long' or 'short'
 
`--ascii`  
      Restricts characters to original 128 ascii set
 
`--ip`  
      View the ip-level dependency graph

## __EXAMPLES__

```
orbit tree --root nor_gate
orbit tree --ip
```