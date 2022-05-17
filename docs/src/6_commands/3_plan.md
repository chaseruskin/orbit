# __orbit plan__

## __NAME__

plan - generate a blueprint file 

## __SYNOPSIS__

```
orbit plan [options]
```

## __DESCRIPTION__

This command will set up the current ip for build processes. It will collect
all necessary files according to their defined fileset into the 
blueprint.tsv file.
  
By default, the top level unit and testbench are auto-detected according to
the current design heirarchy. If there is ambiguity, it will ask the user to
select one of the possibilities when not set as options.
 
The top level unit and top level testbench will be stored in a .env file to
be set during any following calls to the 'build' command.

## __OPTIONS__

`--top <unit>`  
      The top level entity to explicitly define
  
`--bench <tb>`  
      The top level testbench to explicitly define
   
`--plugin <plugin>`  
      A plugin to refer to gather its declared filesets
 
`--build-dir <dir>`  
      The relative directory to place the blueprint.tsv file
 
`--filset <key=glob>...`  
      A glob-style pattern identified by a name to add into the blueprint    
 
`--clean`  
      Removes all files from the build directory before planning
  
`--all`  
      Ignore any design hierarchy and include all hdl files
      
## __EXAMPLES__

```
orbit plan --top top_level --fileset PIN-PLAN="*.board"
```