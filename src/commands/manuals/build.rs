// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    build - execute a plugin/backend tool flow

SYNOPSIS
    orbit build [options] [--] [args]...

DESCRIPTION
    This command will call a user-defined command or plugin. A plugin should
    typically require a blueprint.tsv to be generated. The command also
    should read the data from the blueprint, and then process that data
    (synthesis, simulation, etc.).

OPTIONS
    --plugin <alias>  
          Plugin to execute
     
    --command <cmd>     
          Command to execute
     
    -- args...  
          Arguments to pass to the requested plugin

EXAMPLES
    orbit build xsim -- --waves
";