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

// Automatically generated from the mansync.py script.
pub const HELP: &str = r#"Navigate hdl design unit source code.

Usage:
    orbit read [options] <unit>

Arguments:
    <unit>                primary design unit identifier

Options:
    --ip <spec>           the ip specification to search in the catalog
    --file                copy the source code to a temporary read-only file
    --location            append the targeted code segment's line and column number to the resulting filepath 
    --keep                do not clean the temporary directory of existing files
    --limit <num>         set a maximum number of lines to write
    --start <code>        start the code navigation upon matching this VHDL segment
    --end <code>          stop the code navigation upon matching this VHDL segment
    --doc <code>          navigate to the preceding comments of this VHDL segment

Use 'orbit help read' to read more about the command."#;
