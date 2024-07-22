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
    search - browse the ip catalog

SYNOPSIS
    orbit search [options] [<ip>]

DESCRIPTION
    This command will display a list of all the known ip in the catalog. The catalog
    consists of 3 levels: cache, downloads, and channels.
    
    Any ip at the cache level are considered installed. Any ip at the downloads
    level are considered downloaded. Any ip at the channels level is considered
    available. An ip does not exist in the catalog if it is not found at any one
    of the three defined levels.
    
    When a package name is provided for '<ip>', it will begin to partially match 
    the name with the names of the known ip. If an ip's name begins with '<ip>', it
    is included in the filtered resultes. To strictly match the argument against an
    ip name, use '--match'.

OPTIONS
    <ip>
        The beginning of a package name

    --install, -i
        Filter ip installed to the cache

    --download, -d
        Filter ip downloaded to the downloads

    --keyword <term>...
        Include ip that contain this keyword

    --limit <num>
        The maximum number of results to return

    --match
        Return results that only pass each filter

EXAMPLES
    orbit search axi
    orbit search --keyword memory --keyword ecc
    orbit search --keyword RF --limit 20
"#;
