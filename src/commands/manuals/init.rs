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
    init - initialize an ip from an existing project

SYNOPSIS
    orbit init [options] [<path>]

DESCRIPTION
    This command will initialize a new ip at the target directory '<path>'. If no path
    is supplied, then it defaults to the current working directory.
    
    If no name is supplied, then the ip's name defaults to the final path component
    of the path argument. Use the name option to provide a custom name.
    
    This command fails if the path does not exist. See the 'new' command for
    creating an ip from a non-existing directory.

OPTIONS
    <path>
        The location to initialize an ip

    --name <name>
        The name of the ip

    --library <library>
        The ip library

    --force
        Overwrite a manifest if one already exists

EXAMPLES
    orbit init
    orbit init ./projects/gates
    orbit init --name hello_world
"#;
