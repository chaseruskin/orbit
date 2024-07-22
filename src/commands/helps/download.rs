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
pub const HELP: &str = r#"Fetch packages from the internet.

Usage:
    orbit download [options]

Options:
    --list              print URLs to the console and exit
    --missing           filter only uninstalled packages (default: true)
    --all               include dependencies of all types
    --queue <dir>       set the destination directory to place fetched codebase
    --verbose           display the command being executed
    --force             fallback to default protocol if missing given protocol

Use 'orbit help download' to read more about the command.
"#;
