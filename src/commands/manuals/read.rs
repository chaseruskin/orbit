// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    read - inspect hdl source code files

SYNOPSIS
    orbit read [options] <entity-path>

DESCRIPTION
    This command will ...

OPTIONS
    <entity-path>  
          The pkgid and entity identifier to request [pkgid:]<entity>
     
    --variant, -v <version>  
          Version of ip to fetch
     
    --editor <editor>
          The command to open the requested text-editor

EXAMPLES
    orbit read kepler.rary.gates:and_gate -v 1.0.0
    orbit read :multiplier --editor code
    orbit get multiplier --ip kepler.rary.gate@v1.0.0
    orbit get <unit> --ip kepler.rary.gates";