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

// Automatically generated from the mansync.py script.
pub const HELP: &str = r#"Browse the ip catalog.

Usage:
    orbit search [options] [<ip>]

Options:
    <ip>                  the beginning of a package name
    --install, -i         filter ip installed to the cache
    --download, -d        filter ip downloaded to the downloads
    --keyword <term>...   include ip that contain this keyword
    --limit <num>         the maximum number of results to return
    --match               return results that only pass each filter

Use 'orbit help search' to read more about the command."#;
