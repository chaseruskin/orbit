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
    install - store an immutable reference to a project

SYNOPSIS
    orbit install [options]

DESCRIPTION
    This command will place a project into the cache. By default, the specified
    version is the 'latest' released version of the project that Orbit has 
    identified.
    
    When this command is ran without specifying the '<project>' or a source (such 
    as '--url' or '--path'), it will attempt to install the current project, if it
    exists.
    
    By default, any dependencies required only for development by the target
    project are omitted from installation. To also install these dependencies, 
    use the '--all-deps' flag.
    
    If a protocol is recognized using '--protocol', then an optional tag can also 
    be supplied to help the protocol with providing any additional information it
    may require.
    
    The '--path' option can accept a file system path that is either 1) the root 
    directory that contains the manifest file or 2) a zip archive file that when 
    uncompressed, has the manifest file at the root directoy.
    
    The '--all-public' flag can be used to skip providing the "project.public" 
    field in the current project's manifest, granting the assumption that all 
    source files should be public. It can only be present when installing from a 
    local project and when the "project.public" field does not exist.
    
    To remove a project from the catalog, see the 'remove' command.

OPTIONS
    <project>
        Project id specification

    --url <url>
        Url to install the project from the internet

    --path <path>
        Path to install the project from local file system

    --protocol, -p <name>
        Protocol to download the project

    --force
        Install the project regardless of the cache slot occupancy

    --offline
        Skip checking coherency with repository

    --list, -l
        View available protocols and exit

    --all-deps
        Install all dependencies (including development)

    --all-public
        Install with all source files being public

EXAMPLES
    orbit install
    orbit install lcd_driver:2.0
    orbit install adder:1.0.0 --url https://my.adder/project.zip
    orbit install alu:2.3.7 --path ./projects/alu --force
"#;
