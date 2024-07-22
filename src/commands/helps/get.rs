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
pub const HELP: &str = r#"Fetch an hdl entity for code integration.

Usage:
    orbit get [options] <unit>

Args:
    <unit>                  entity identifier

Options:
    --ip <spec>             ip to reference the unit from
    --json                  export the entity information as valid json
    --library,   -l         display library declaration
    --component, -c         display component declaration
    --signals,   -s         display constant and signal declarations
    --instance,  -i         display instantation
    --architecture, -a      display detected architectures
    --name <identifier>     set the instance's identifier
    --signal-prefix <value> prepend information to the instance's signals
    --signal-suffix <value> append information to the instance's signals

Use 'orbit help get' to read more about the command.
"#;
