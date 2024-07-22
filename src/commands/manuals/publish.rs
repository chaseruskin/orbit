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
    publish - post an ip to a channel

SYNOPSIS
    orbit publish [options]

DESCRIPTION
    Performs a series of checks on a stable version of a local ip to then release it
    through a channel.
    
    For an ip to be published, it must have its source field defined that directs to
    a valid internet location.
    
    By default, it operates a dry run, performing all steps in the process except
    for the actual release through the channel. To fully run the command, use the
    '--ready' flag. When the ip is published, it will also be installed to the cache
    by default. To skip this behavior, use the '--no-install' flag.

OPTIONS
    --ready, -y
        Perform a full run

    --no-install
        Skip installing the ip

    --list
        View available channels and exit

EXAMPLES
    orbit publish
    orbit publish --ready
"#;
