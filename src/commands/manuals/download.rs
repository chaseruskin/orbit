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
    download - fetch packages from the internet

SYNOPSIS
    orbit download [options]

DESCRIPTION
    This command will get a package from the internet using the default protocol
    or a user-defined protocol. It places the package in the path defined as
    environment variable '$ORBIT_ARCHIVE'.
    
    Downloads are vital to Orbit's management system as to avoid having to solely 
    rely on accessing the internet to get IP. Downloads allow Orbit to quickly
    repair broken installations and allow users to learn about IP before installing.
    
    When using a custom protocol, Orbit expects the final fetched repository to
    exist within a special directory called the queue. By default, the queue is set
    to a temporary directory, but it can be overridden with '--queue'. After a 
    protocol is executed, Orbit resumes the download process by trying to detect the 
    target IP and then performing a compression algorithm on the path to store as a 
    single file archive. This final archive is special and packed with additional 
    bytes, which makes it unsuitable to easily unzip with existing compression 
    tools.
    
    A lockfile is required to exist in the current IP in order to download its 
    dependencies.
    
    Variable substitution is supported when specifying the "command" and "args"
    fields for a protocol. Most notably, the queue is accessed as 
    '{{ orbit.queue }}'. See 'orbit help protocols' for more information about 
    available variables.
    
    This action may automatically run during an install if the package is missing
    from the downloads. See 'orbit help install' for more details.

OPTIONS
    --list
        Print urls and exit

    --missing
        Filter only uninstalled packages (default: true)

    --all
        Gather packages from all dependency types

    --queue <dir>
        Set the destination directory for placing fetched repositories

    --verbose
        Display the custom protocol being executed

    --force
        Download selected packages regardless of status

EXAMPLES
    orbit download --missing --list
    orbit download --all --force
"#;
