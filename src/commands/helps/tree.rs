//
//  Copyright (C) 2022-2025  Chase Ruskin
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
pub const HELP: &str = r#"Show a dependency graph.

Usage:
    orbit tree [options] [<unit>...]

Options:
    <unit>...             uppermost HDL unit of the dependency tree
    --edges, -e <kind>    the kind of dependencies to display (unit, project, all)
    --format <format>     determine how to display node names (short, long)
    --no-dedupe           do not de-duplicate repeated dependencies
    --charset <charset>   choose the character set for the tree (utf8, ascii)
    --json                export the tree's information as valid json

Use 'orbit help tree' to read more about the command."#;
