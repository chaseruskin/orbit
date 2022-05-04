// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    launch - release an ip's next version 

SYNOPSIS
    orbit launch [options] <version>

DESCRIPTION
    This command will tag the current commit as a new version. By default, it
    will perform a dry run of the launch process to verify the procedure will 
    run with no errors. To actually perform a launch, include the '--ready'
    flag.   

OPTIONS
    --ready  
      perform a real run through the launch process

EXAMPLES
    orbit launch 1.0.0
    orbit launch 1.0.0 --ready
";