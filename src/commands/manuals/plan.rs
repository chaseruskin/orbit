// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    plan - generate a blueprint file 

SYNOPSIS
    orbit plan [options]

DESCRIPTION
    This command will set up the current ip for build processes. It will collect
    all necessary files according to their defined fileset into the 
    blueprint.tsv file.
      
    By default, the top level unit and testbench are auto-detected according to
    the current design heirarchy. If there is ambiguity, it will ask the user to
    select one of the possibilities when not set as options.
     
    The top level unit and top level testbench will be stored in a .env file to
    be set during any following calls to the 'build' command.

OPTIONS
    --top <unit>  
          The top level entity to explicitly define
      
    --bench <tb>  
          The top level testbench to explicitly define
       
    --plugin <alias>  
          A plugin to refer to gather its declared filesets
     
    --build-dir <dir>  
          The relative directory to place the blueprint.tsv file
     
    --filset <key=glob>...  
          A glob-style pattern identified by a name to add into the blueprint    
     
    --clean  
          Removes all files from the build directory before planning
      
    --list  
          Display all available plugins and exit
     
    --all  
          Ignore any design hierarchy and include all hdl files
     
    --disable-ssh  
          Convert SSH to HTTPS urls when fetching external dependencies
          

EXAMPLES
    orbit plan --top top_level --fileset PIN-PLAN=\"*.board\"
";