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

use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::{error::Error, util::anyerror::Fault};

pub type Channels = Vec<Channel>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sequence {
    command: String,
    args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    name: String,
    description: Option<String>,
    /// The directory located where the channel exists.
    path: Option<String>,
    sync: Option<Sequence>,
    pre: Option<Sequence>,
    post: Option<Sequence>,
    /// Run command sequences from this directory and place manifest data here during launch
    #[serde(skip_serializing, skip_deserializing)]
    root: Option<PathBuf>,
}

impl Channel {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_root(&self) -> &PathBuf {
        self.root.as_ref().unwrap()
    }

    /// Displays a plugin's information in a single line for quick glance.
    pub fn quick_info(&self) -> String {
        format!(
            "{:<24}{}",
            self.name,
            self.description.as_ref().unwrap_or(&String::new())
        )
    }

    /// Resolves the root path according to its path.
    pub fn set_root(&mut self, relative_from: PathBuf) -> Result<(), Fault> {
        match &self.path {
            Some(p) => {
                let p = PathBuf::from(p);
                let fp = if p.is_relative() == true {
                    relative_from.join(p)
                } else {
                    p
                };
                if fp.exists() == false {
                    return Err(Error::ChannelPathNotFound(fp))?;
                }
                if fp.is_dir() == false {
                    return Err(Error::ChannelPathNotDir(fp))?;
                }
                self.root = Some(fp);
            }
            None => {
                self.root = Some(relative_from);
            }
        }
        Ok(())
    }
}

impl Channel {
    /// Creates a string to display a list of channels.
    ///
    /// The string lists the channels in alphabetical order by `alias`.
    pub fn list_channels(chans: &mut [&&Channel]) -> String {
        let mut list = String::new();
        chans.sort_by(|a, b| a.name.cmp(&b.name));
        for c in chans {
            list += &format!("{}\n", c.quick_info());
        }
        list
    }
}
