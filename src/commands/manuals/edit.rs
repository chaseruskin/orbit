// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    edit - develop an ip in a text editor

SYNOPSIS
    orbit edit [options]

DESCRIPTION
    This command will open an ip project in the configured text editor. To
    determine the editor, it will first check if the EDITOR environment
    variable is set. If not, it will search for the key 'core.editor' in the
    config.toml file. Explicitly setting the '--editor' option will override
    any previously determined value.
      
    The ip's project directory must be located on the DEV_PATH. The ip's 
    project path will be passed as an argument to the text editor command. For 
    example, if EDITOR=\"code\", then the command orbit will execute is: 
    code <ip-path>.
      
    By default, the edit command is set to use the 'open' mode. This mode
    requires an editor to be set and will invoke it as a subprocess. Selecting
    the 'path' mode does not require an editor value and will display the path
    to the requested directory/file to edit.

OPTIONS
    --ip <pkgid>  
          The ip to find in the development state
      
    --editor <editor>  
          The command to open the requested text-editor
      
    --config  
          Modify the global configuration file
      
    --mode <mode>  
          Select how to edit: 'open' or 'path'

EXAMPLES
    orbit edit --ip ks-tech.rary.gates --editor=code
    orbit edit --config --mode path
";