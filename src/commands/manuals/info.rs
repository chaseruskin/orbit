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
    info - display information about a project

SYNOPSIS
    orbit info [options] [<project>]

DESCRIPTION
    Displays various bits of information about a particular project. If no project 
    is provided, then it displays information related to the local project.
    
    To display manifest information, no additional options are required.
    
    To display the available units within the project, use the '--units' option. 
    For non-local projects, its protected and private design units are hidden
    from the results. To include design units of all visibility levels, use the 
    '--force' flag.
    
    To display the known versions for a project, use the '--versions' option.

OPTIONS
    <project>
        Project id specification

    --versions, -v
        Display the list of known versions

    --units, -u
        Display the units for this project

EXAMPLES
    orbit info --units
    orbit info gates:1.0.0 --units --force
    orbit info gates --versions
    orbit info gates:1 --versions
"#;
