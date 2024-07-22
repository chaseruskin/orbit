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

// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Inspect hdl design unit source code.

Usage:
    orbit read [options] <unit>

Args:
    <unit>                  primary design unit identifier

Options:            
    --ip <spec>             ip to reference the unit from
    --location              append the :line:col to the filepath
    --file                  display the path to the read-only source code
    --keep                  prevent previous files read from being deleted
    --limit <num>           set a maximum number of lines to print
    --start <code>          tokens to begin reading contents from file
    --end <code>            tokens to end reading contents from file
    --doc <code>            series of tokens to find immediate comments for

Use 'orbit help read' to read more about the command.
"#;
