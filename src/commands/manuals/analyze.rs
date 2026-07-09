//
//  Copyright (C) 2022-2026  Chase Ruskin
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
    analyze - print a blueprint

SYNOPSIS
    orbit analyze [options]

DESCRIPTION
    Prints the blueprint to stdout.
    
    This command performs the planning stage of the build process and displays
    the resulting blueprint contents to stdout. Entries in the blueprint are
    guaranteed to be in a deterministic order.
    
    If the '--plan' option is omitted, then the default plan is used (tsv).
    
    If '--local' is used, then only the entries for source files belonging to
    the current project are included in the final blueprint. If the blueprint
    includes dependency information from a different project, the dependency 
    information is still included for that entry.
    
    If '--force' is used, then a blueprint will be displayed (albeit possibly
    incomplete) regardless if there are HDL parsing or analysis errors.

OPTIONS
    --local
        Filter entries that belong to the working project

    --force
        Force the blueprint to be displayed

    --plan <format>
        Set the blueprint file format

EXAMPLES
    orbit analyze --plan json --local
"#;
