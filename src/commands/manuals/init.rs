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
    Initializes an ip at the file system directory '<path>'. If not path is
    provided, then it defaults to the current working directory. 
    
    If no name is provided, then the resulting ip's name defaults to the 
    directory's name. Using the '--name' option allows the ip's name to be 
    explicitly set.
    
    To create a new ip from a non-existing directory, see the 'new' command.

OPTIONS
    <path>
        Directory to initialize

    --name <name>
        Set the resulting ip's name

    --lib <lib>
        Set the resulting ip's library

EXAMPLES
    orbit init
    orbit init projects/gates
    orbit init --name adder
"#;
