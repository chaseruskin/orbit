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
pub const HELP: &str = r#"Modify configuration values.

Usage:
    orbit config [options]

Options:
    --global                    access the home configuration file
    --local                     access the current project configuration file
    --append <key>=<value>...   add a value to a key storing a list
    --pop <key>...              remove the last value to a key storing a list
    --set <key>=<value>...      write the value at the key entry
    --unset <key>...            delete the key's entry

Use 'orbit help config' to read more about the command.
"#;
