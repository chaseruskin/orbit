// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    get - fetch an hdl entity for code integration

SYNOPSIS
    orbit get [options] <unit>

DESCRIPTION
    This command will provide the relevant information about the requested HDL
    entity required to integrate the code into the current design. The command
    produces valid HDL code displayed to stdout that allows a user to copy and
    paste the results into a new hdl source code file for proper hierarchy code
    reuse.
    
    If the spec if not provided with '--ip', then it will search the current
    working ip for the requested HDL entity.
    
    If the '--instance' flag is used without the '--component' flag, then it will
    display the direct instantiation style code for VHDL (VHDL-93 feature).
    
    It is important to note that any units referenced from ip outside of the
    current working ip are not automatically tracked as a dependency. In order to
    add an ip as a dependency to properly reference its source code files, edit
    the current working ip's manifest with a new entry under the '[dependencies]'
    table with the dependency ip and its version.

OPTIONS
    <unit>
        Primary design unit identifier

    --ip <spec>
        The ip that contains the requested unit

    --json
        Export the entity information as valid json

    --component, -c
        Display the component declaration

    --signals, -s
        Display the constant and signal declarations

    --instance, -i
        Display the unit's instantiation

    --architecture, -a
        Display the detected architectures

    --name <identifier>
        Set the instance's identifier

EXAMPLES
    orbit get and_gate --ip gates:1.0.0 --component
    orbit get ram --ip mem:2.0.3 -csi
    orbit get uart -si --name u0
    orbit get or_gate --ip gates --json
"#;
