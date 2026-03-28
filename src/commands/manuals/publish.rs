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
    publish - post a project to a channel

SYNOPSIS
    orbit publish [options]

DESCRIPTION
    Performs a series of checks for a local project and then releases it to its 
    specified channel(s).
    
    There are multiple checks that are performed before a project can be published. 
    First, the project must have an up to date lockfile with no relative 
    dependencies. The project's manifest must also have a value for the source 
    field. In addition, Orbit must be able to construct the HDL source code graph 
    without errors. Finally, the project is downloaded from its source url and 
    temporarily installed to verify its contents match those of the local project.
    
    Posting a project to a channel involves copying the project's manifest file to 
    a path within the channel known as the index. For every publish of a project, 
    the index corresponds to a unique path within the channel that gets created by 
    Orbit. A channel's pre-publish and post-publish hooks can get the value for the
    project's index by reading the ORBIT_CHANNEL_PROJECT_DIR environment variable.
    
    The '--all-public' flag can be used to skip providing the "project.public" 
    field in the current project's manifest, granting the assumption that all 
    source files should be public. It cannot be used if the "project.public" field 
    exists.
    
    If at least one channel is specified on the command-line using the '--channel'
    option, then any channel list found in the project's manifest under the 
    "project.channels" field will be ignored as well as any channel list found in the
    "publish.default-channels" field of a configuration file.
    
    By default, this command performs a dry run, which executes all of the steps 
    in the process except for actually posting the project to its channel(s). 
    To run the command to completion, use the '--ready' option.
    
    If the publishing process fails during the channel's pre or post commands,
    Orbit will rollback its changes by removing any files it copied into the
    channel as well as any newly created directories it made.

OPTIONS
    --ready, -y
        Run the operation to completion

    --channel, -c <name>...
        Use this channel during publishing

    --no-install
        Do not install the project for future use

    --list, -l
        View available channels and exit

    --all-public
        Publish with all source files being public

EXAMPLES
    orbit publish
    orbit publish --ready
    orbit publish -c hyperspace-labs -y
"#;
