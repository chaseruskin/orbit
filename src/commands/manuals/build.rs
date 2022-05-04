// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    build - execute a plugin/backend tool flow

SYNOPSIS
    orbit build [options] <plugin> [--] [args]...

DESCRIPTION
    This command will call a user-defined command (plugin). A plugin should
    typically require a blueprint.tsv to be generated. The command also
    should read the data from the blueprint, and then process the data 
    (synthesis, simulation, etc.).

OPTIONS
    -- args...  
       Arguments to pass to the requested command

EXAMPLES
    orbit build xsim -- --waves
";