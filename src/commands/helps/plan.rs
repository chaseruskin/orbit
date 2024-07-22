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
pub const HELP: &str = r#"Prepare a target.

Usage:
    orbit plan [options]              

Options:
    --top <unit>            override auto-detected toplevel entity
    --bench <unit>            override auto-detected toplevel testbench
    --target <name>        collect filesets defined for a target
    --target-dir <dir>       set the output build directory
    --fileset <key=glob>... set an additional fileset
    --clean                 remove all files from the build directory
    --list                  view available targets and exit
    --lock-only             create the lockfile and exit
    --all                   include all found HDL files
    --force                 skip reading from the lock file

Use 'orbit help plan' to read more about the command.
"#;
