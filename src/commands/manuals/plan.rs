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
    the current design heirarchy. If there is ambiguity, it will show the user
    the possibilities.
      
    The top level unit and top level testbench will be stored in a .env file to
    be set during any following calls to the 'build' command. If a plugin was
    specified, it will also be stored in the .env file to be recalled during the
    building phase.
      
    User-defined filesets are only collected along the current working ip's 
    path. Specifying a plugin with --plugin will collect the filesets 
    configured for that plugin.
      
    During the planning phase, a lockfile is produced outlining the exact ip
    dependencies required, how to get them, and how to verify them. The lockfile
    should be checked into version control and not directly edited by the user.
      
    If the current working ip's manifest's data matches its data stored in its
    own lockfile, then Orbit will read from the lockfile to create the ip
    dependency graph. To force Orbit to build the ip dependency graph from
    scratch, use --force.
      
    If only wishing to update the lockfile, using --lock-only will not require
    a toplevel or testbench to be determined. The --lock-only flag can be
    combined with --force to overwrite the lockfile regardless if it is
    already in sync with the current working ip's manifest data.

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
     
    --force  
          Ignore reading the lock file
      
    --lock-only
          Only create a lock file

EXAMPLES
    orbit plan --top top_level --fileset PIN-PLAN=\"*.board\"
    orbit plan --plugin vivado --clean --bench ram_tb
    orbit plan --lock-only
";
