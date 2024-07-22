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
pub const HELP: &str = r#"Browse the ip catalog.

Usage:
    orbit search [options] [<ip>]

Args:
    <ip>                filter the name of ip

Options:
    --install, -i       filter ip installed to cache
    --download, -d      filter ip downloaded to downloads
    --keyword <term>... special word to filter out packages
    --limit <num>       maximum number of results to return
    --match             only return results with each filter passed

Use 'orbit help search' to read more about the command.
"#;
