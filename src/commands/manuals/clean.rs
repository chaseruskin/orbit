//
//  Copyright (C) 2022-2026  Chase Ruskin
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
    clean - remove the target directory

SYNOPSIS
    orbit clean [options]

DESCRIPTION
    Remove artifacts that Orbit has generated in the past.
    
    By default, this command deletes all directories found within the current
    project's target directory. To select a single target's directory for removal,
    use the '--target' option.

OPTIONS
    --doc
        Whether or not to clean just the documentation directory

    --target, -t <name>
        Target output directory to just remove

    --target-dir <dir>
        The relative directory where the targets exist

EXAMPLES
    orbit clean
    orbit clean --target modelsim
"#;
