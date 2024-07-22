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
    plan - prepare a target

SYNOPSIS
    orbit plan [options]

DESCRIPTION
    This command carries out the "planning phase". This phase involves reading
    and writing to the lockfile and collecting all necessary files according to 
    their defined fileset into a blueprint file for future build processes. This 
    command acts upon the current working ip.
    
    By default, the top level unit and testbench are auto-detected according to
    the current design heirarchy. If there are multiple candidates for a potential
    top level or testbench, it will exit and ask the user to explicitly select
    a candidate. To include all top levels and testbenches, use '-all'.
    
    The top level unit and testbench will be stored in a '.env' file within the
    build directory. This '.env' file is read during the build command to set
    the proper environment variables for downstream targets and scripts that
    may require this information. If a known target is provided with '--target',
    then it will also be stored in the '.env' file to be recalled during the
    build phase.
    
    User-defined filesets are only collected within the current working ip's 
    path. Targets may have custom filesets defined in their configuration. When
    specifying a known target with '--target', it will collect the filesets 
    defined for that target. Use '--fileset' as many times as needed to define
    additional filesets.
    
    During the planning phase, a lockfile is produced outlining the exact ip
    dependencies required, how to get them, and how to verify them. The lockfile
    should be checked into version control and should not manually edited by the 
    user.
    
    If the current working ip's manifest data matches its data stored in its
    own lockfile, then Orbit will read from the lockfile to create the ip
    dependency graph. To force Orbit to build the ip dependency graph from
    scratch, use '--force'.
      
    If only needing to update the lockfile, use '--lock-only'. This flag does 
    not require a toplevel or testbench to be determined. Using '--lock-only' with
    '--force' will overwrite the lockfile regardless if it is already in sync 
    with the current working ip's manifest data.
    
    When updating the lockfile, this command will download and install any new
    dependencies if necessary. To only download an ip, see the 'download' command.
    To only install an ip, see the 'install' command.
    
    If an installed dependency's computed checksum does not match the checksum
    stored in the lockfile, it assumes the installation to be corrupt and will 
    re-install the dependency to the cache.

OPTIONS
    --top <unit>
        The top level entity to explicitly define

    --bench <unit>
        The top level testbench to explicitly define

    --target <name>
        A target to refer to gather its declared filesets

    --target-dir <dir>
        The relative directory to place the blueprint.tsv file

    --fileset <key=glob>...
        A glob-style pattern identified by a name to add into the blueprint

    --clean
        Removes all files from the build directory before execution

    --list
        Display all available targets and exit

    --force
        Ignore reading the precomputed lock file

    --lock-only
        Create the lock file and exit

    --all
        Include all locally found HDL files

EXAMPLES
    orbit plan --bench my_tb
    orbit plan --top and_gate --fileset PIN-PLAN="*.board"
    orbit plan --target vivado --clean --bench ram_tb
    orbit plan --lock-only
"#;
