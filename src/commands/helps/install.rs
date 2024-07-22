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

// This help page was automatically generated from the mangen.py tool.
pub const HELP: &str = r#"Store an immutable reference to an ip.

Usage:
    orbit install [options]

Options:
    <ip>                ip specification to install from catalog
    --url <url>         URL to install the ip from the internet
    --path <path>       ip's local path to install from filesystem
    --protocol <name>   defined protocol to download the package
    --tag <tag>         unique tag to pass to the protocol
    --all               install all dependencies including development
    --list              view available protocols and exit
    --verbose           display the command(s) being executed
    --force             install regardless of cache slot occupancy

Use 'orbit help install' to read more about the command.
"#;
