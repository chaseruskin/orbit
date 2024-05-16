// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    remove - uninstall an ip from the catalog

SYNOPSIS
    orbit remove [options] <ip>

DESCRIPTION
    This command will remove known ip stored in the catalog. By default, it will
    remove the ip from the cache. This include any dynamic entries spawned from the
    requested ip to remove.
    
    To remove the ip from the cache and downloads locations, use '--all'.

OPTIONS
    <ip>
        Ip specification

    --all
        remove the ip from the cache and downloads

    --recurse
        fully remove the ip and its dependencies

EXAMPLES
    orbit remove gates
    orbit remove gates:1.0.0 --all
"#;
