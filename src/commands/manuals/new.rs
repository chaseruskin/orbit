// This manual page was automatically generated from the rsmangen.py tool.
pub const MANUAL: &str = "\
NAME
    new - create a new ip

SYNOPSIS
    orbit new [options]

DESCRIPTION
    This command will create a new ip project. The default destination path is
    the vendor/library/name relative to the DEV_PATH. If the DEV_PATH is not
    configured, then it will use the current working directory. Creating a new 
    ip involves creating a manifest file Orbit.toml and initializing an empty
    git repository.
      
    Use --to to override the destination path. This path is not allowed to
    exist unless --force is specified.
      
    Copying from existing files can be achieved in two ways. The recommended way
    is to configure templates, which can be viewed with --list. Using 
    --template will import the files from the template's root directory when
    creating a new ip. On the other hand, using --from will import files from 
    that directory.
      
    Upon creation of an ip or file, variable substitution is performed. Variable
    substitution takes form as a template processor using known information
    about orbit's state and injecting into templated files.
      
    A new file is able to be generated from within an ip under development with
    the --file flag. You can view available files for importing from a
    particular template by combining options --template and --list. To use
    a file from a template in creating a new file, specify the template and
    the source file's relative path with --template and --from. You can
    specify a source path not tied to a template by just using --from.
       
    If --from is omitted when creating a file, an empty file will be created.

OPTIONS
    --ip <pkgid>  
          The vendor.library.name for the new project
     
    --template <alias>  
          Specify a configured template to import
     
    --to <path>  
          Specify the destination path
     
    --file  
          Create a file for the current ip
     
    --from <path>  
          Specify the source path to copy
     
    --list  
          View available templates or files within a specified template
     
    --force  
          Overwrites the destination path if it already exists

EXAMPLES
    orbit new --list
    orbit new --ip ks-tech.rary.gates --to gates --template base
    orbit new --template base --list
    orbit new --file --to rtl/circuit2.vhd --template base --from extra/cmb.vhd
    orbit new --ip ks-tech.util.toolbox --from ../template
";
