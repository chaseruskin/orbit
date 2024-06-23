// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    view - display metadata of an ip

SYNOPSIS
    orbit show [options] [<ip>]

DESCRIPTION
    This command retrieves various pieces of information about a particular ip to
    gain a better understanding of how to utilize the ip. By default, it displays
    the ip's manifest, if and only if the ip is able to be located.
    
    It will first attempt to return the information from a possible installation. If
    one does not exist, then it searches the downloads location for the ip.
    
    If '--units' is specified, then a list of the ip's HDL units are displayed.
    
    If '--versions' is specified, then a list of the ip's already available versions
    are displayed.
    
    If no spec is provided for '<ip>', then it will retrieve information based on the
    current working ip, if exists.

OPTIONS
    <ip>
        The spec of the ip to query

    --versions, -v
        Display the list of possible versions

    --units, -u
        Display the list of HDL primary design units associated with this ip

EXAMPLES
    orbit view --units
    orbit view gates:1.0.0 -u
    orbit view gates --versions
"#;
