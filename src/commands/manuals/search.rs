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

// This manual page was automatically generated from the mangen.py tool.
pub const MANUAL: &str = r#"NAME
    search - browse the catalog

SYNOPSIS
    orbit search [options] [<project>]

DESCRIPTION
    Returns a list of the projects found in the catalog.
    
    The returned list of projects is organized by row with each row consisting of four
    columns: the project's name, latest version, catalog state, and UUID.
    
    By default, all projects in the catalog will be returned. To filter by project
    name, use the '<project>' option. To limit the number of results, use the 
    '--limit' option.
    
    A project can be stored across three different levels: installed in the cache,
    downloaded to the archive, and available via channels. By default, all levels
    are searched for projects. Applying a level filter ('--install', '--download', 
    '--available' options) will restrict the search to only checking the filtered
    levels for projects.
    
    A resulting project is only read from one level, even when multiple levels are
    searched. When a project exists at multiple levels, the catalog imposes a 
    priority on which level to choose. Installed projects have higher priority 
    over downloaded projects, and downloaded projects have higher priority over 
    available projects.
    
    Results can also be filtered by keyword using the '--keyword' option. By
    default, if a project matches at least one filter then it will be returned in 
    the result. To collect only projects that match each presented filter, use 
    the '--match' option.
    
    If a project has a higher version that exists and is not currently installed, 
    then an asterisk character "*" will appear next the project's version. To 
    update the project to the latest version, see the 'install' command.

OPTIONS
    <project>
        Project id specification

    --install, -i
        Filter projects installed to the cache

    --download, -d
        Filter projects downloaded to the archive

    --available, -a
        Filter projects available via channels

    --keyword <term>...
        Include projects that have this keyword

    --limit <n>
        Maximum number of results to return

    --match
        Return results that pass each filter

EXAMPLES
    orbit search axi
    orbit search --keyword memory --keyword ecc
    orbit search --keyword cdc --limit 20 -i
"#;
