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
    read - navigate hdl design unit source code

SYNOPSIS
    orbit read [options] <unit>

DESCRIPTION
    This command allows the user to navigate source code to gain a quicker
    understanding of the available code. By default, it will display the code to the
    console.
    
    If no ip specification is provided through the '--ip' option, then it will
    assume to search the current working ip, if it exists.
    
    If '--file' is provided, then the source code will be written to a temporary
    read-only file. Also providing '--location' in this context will append the
    requested code segment's line and column number to the end of the generated
    filepath.
    
    The options '--start', '--end', and '--doc' all accept valid VHDL code to
    search for in the identified source code file. The '--doc' option will find the
    immediate single-line comments preceding the supplied code value.
    
    The 'read' command attempts to clean the temporary directory at every call to
    it. To keep existing files alive while allowing new files to appear, use the
    '--keep' flag.

OPTIONS
    <unit>
        Primary design unit identifier

    --ip <spec>
        The ip specification to search in the catalog

    --file
        Copy the source code to a temporary read-only file

    --location
        Append the targeted code segment's line and column number to the resulting filepath 

    --keep
        Do not clean the temporary directory of existing files

    --limit <num>
        Set a maximum number of lines to write

    --start <code>
        Start the code navigation upon matching this vhdl segment

    --end <code>
        Stop the code navigation upon matching this vhdl segment

    --doc <code>
        Navigate to the preceding comments of this vhdl segment

EXAMPLES
    orbit read and_gate --ip gates:1.0.0
    orbit read math_pkg --ip math --doc "function clog2"
    orbit read math_pkg --ip math --start "package math_pkg" --doc "function flog2p1" --location --file
"#;
