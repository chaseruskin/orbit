//
//  Copyright (C) 2022-2025  Chase Ruskin
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
    doc - generate a project's documentation

SYNOPSIS
    orbit doc [options]

DESCRIPTION
    Generates documentation for a project and all of its dependencies based on
    inline markdown comments.
    
    The BROWSER environment variable is required to be set when using the '--open'
    flag.

OPTIONS
    --open
        Open the docs in a browser after the operation

    --no-deps
        Don't build documentation for dependencies

    --document-private-items
        Document private items

    --target-dir <dir>
        Directory for all generated artifacts and intermediate files

EXAMPLES
    orbit doc
    orbit doc --no-deps
"#;
