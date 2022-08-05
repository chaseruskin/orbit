// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    read - inspect hdl source code files

SYNOPSIS
    orbit read [options] <unit>

DESCRIPTION
    This command will open a primary design units source code file for
    examination in a text editor. If the unit is from the current ip, then it
    will return the direct reference to the file. If the unit comes from 
    outside the current ip, then Orbit will create a read-only copy of the 
    file in a temporary directory.
      
    By default, previous files in the temporary directory are removed on every
    new call to this command. If you are viewing multiple files at the same,
    use --no-clean to keep previous files being read existing.
      
    The line and column where the primary design unit code is found can be
    appended to the source code file path using --location. The syntax for
    the filepath becomes <file>:<line>:col.
      
    By default, the mode to read a source code file is 'open', which will pass
    the source code filepath as an argument to the configured editor. If the
    mode is 'path', then the editor is ignored and the filepath is output to
    stdout.

OPTIONS
    <unit>  
          The primary design unit identifier to access
     
    --ip, <pkgid>  
          The ip where the unit is existing
     
    --variant, -v <version>  
          Version of ip to fetch
     
    --editor <editor>  
          The command to invoke a text editor
     
    --location  
          Appends the :line:col to the end of the source code file path
     
    --mode <mode>  
          Select how to read the file: options are 'open' or 'path'
     
    --no-clean  
          Keep previously read files existing

EXAMPLES
    orbit read and_gate --ip ks-tech.rary.gates -v 1.0.0
    orbit read multiplier --editor code --no-clean
";