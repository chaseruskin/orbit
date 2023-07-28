// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    build - execute a plugin/backend tool flow

SYNOPSIS
    orbit build [options] [--] [args]...

DESCRIPTION
    This command will call a user-defined command or plugin. A plugin should
    typically require a blueprint.tsv to be generated. The command also
    should read the data from the blueprint, and then process that data
    (synthesis, simulation, etc.).
      
    If the previous plan command accepted a plugin option, then Orbit remembers
    for future build commands. It will be the default plugin to use if no
    '--plugin' or '--command' is entered for the given command.
      
    The command invoked will be ran from the ip's root directory.

OPTIONS
    --plugin <alias>
        Plugin to execute

    --command <cmd>
        Command to execute

    --list
        View available plugins

    --build-dir <dir>
        The relative directory to locate the blueprint.tsv file

    --verbose
        Display the command being executed

    -- args...
        Arguments to pass to the requested plugin

EXAMPLES
    orbit build --plugin xsim -- --waves
    orbit build --command python -- ./tools/synth.py --part x70
    orbit build --verbose
"#;