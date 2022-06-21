// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    edit - further develop an ip in a text editor

SYNOPSIS
    orbit edit [options] <ip>

DESCRIPTION
    This command will open an ip project in the configured text editor. To
    determine the editor, it will first check if the EDITOR environment
    variable is set. If not, it will search for the key 'core.editor' in the
    config.toml file. Explicitly setting the '--editor' option will override
    any previously determined value.
      
    The ip's project directory must be located on the ORBIT_PATH. The ip's 
    project path will be passed as an argument to the text editor command. For 
    example, if EDITOR=\"code\", then the command orbit will execute is: 
    code <ip-path>.

OPTIONS
    <ip>  
          The PKGID to look up the ip under ORBIT_PATH.
      
    --editor <cmd>  
          The command to open the requested text-editor.

EXAMPLES
    orbit edit ks-tech.rary.gates --editor=code
";