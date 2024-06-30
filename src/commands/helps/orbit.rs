// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Orbit is an hdl package manager and build tool.

Usage:
    orbit [options] [command]

Commands:
    new             create a new ip
    init            initialize an ip from an existing project
    view            display metadata of an ip
    read            navigate hdl design unit source code
    get             fetch an entity
    tree            view the dependency graph
    lock            save the world state of the current ip
    test, t         run a test
    build, b        plan and execute a target
    launch          verify an upcoming release
    search          browse the ip catalog 
    download        request packages from the internet
    install         store an immutable reference to an ip
    env             print orbit environment information
    config          modify configuration values
    remove          uninstall an ip from the catalog

Options:
    --version       print version information and exit
    --upgrade       check for the latest orbit binary
    --force         bypass interactive prompts
    --color <when>  coloring: auto, always, never
    --help, -h      print help information

Use 'orbit help <command>' for more information about a command.
"#;
