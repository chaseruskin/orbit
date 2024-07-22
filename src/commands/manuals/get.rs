//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

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
    
    An identifier prefix or suffix can be attached to the signal declarations and
    the instantiation's port connection signals by using '--signal-prefix' and 
    '--signal-suffix' respectively. These optional texts are treated as normal
    strings and are not checked for correct syntax.
    
    When no output options are specified, this command by default will display the
    entity's component declaration.

OPTIONS
    <unit>
        Primary design unit identifier

    --ip <spec>
        The ip that contains the requested unit

    --json
        Export the entity information as valid json

    --library, -l
        Display the unit's library declaration

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

    --signal-prefix <value>
        Prepend information to the instance's signals

    --signal-suffix <value>
        Append information to the instance's signals

EXAMPLES
    orbit get and_gate --ip gates:1.0.0 --component
    orbit get ram --ip mem:2.0.3 -csi
    orbit get uart -si --name u0
    orbit get or_gate --ip gates --json
"#;
