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
    remove - uninstall an ip from the catalog

SYNOPSIS
    orbit remove [options] <ip>

DESCRIPTION
    This command will remove known ip stored in the catalog. By default, it will
    remove the ip from the cache. This include any dynamic entries spawned from the
    requested ip to remove.
    
    To remove the ip from the cache and downloads locations, use '--all'.

OPTIONS
    <ip>
        Ip specification

    --all
        Remove the ip from the cache and downloads

    --recurse, -r
        Fully remove the ip and its dependencies

EXAMPLES
    orbit remove gates
    orbit remove gates:1.0.0 --all
"#;
