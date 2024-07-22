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
    new - create a new ip

SYNOPSIS
    orbit new [options] <path>

DESCRIPTION
    This command will create a new ip at the target directory '<path>'. The command
    assumes the path does not already exists. It will attempt to create a new 
    directory at the destination with a manifest. 
    
    If no name is supplied, then the ip's name defaults to the final path component
    of the path argument. Use the name option to provide a custom name.
    
    This command fails if the path already exists. See the 'init' command for
    initializing an already existing project into an ip.

OPTIONS
    <path>
        The new directory to make

    --name <name>
        The ip name to create

    --library <library>
        The ip library

EXAMPLES
    orbit new gates
    orbit new ./projects/dir7 --name adder
"#;
