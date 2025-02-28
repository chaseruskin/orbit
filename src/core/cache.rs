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

use crate::core::ip::Ip;
use crate::util::anyerror::Fault;
use crate::util::filesystem;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct PkgCache {
    protected: Vec<String>,
}

impl PkgCache {
    /// Creates all the desired cached contents to be stored alongside an installed ip.
    pub fn from_ip(ip: &Ip) -> Result<Self, Fault> {
        // generate the unit map
        let umap = ip.collect_units(false, true)?;
        let protected: Vec<String> = umap
            .iter()
            .filter(|(_, v)| v.get_visibility().is_protected())
            .map(|(_, v)| {
                filesystem::into_std_str(filesystem::remove_base(
                    ip.get_root(),
                    &PathBuf::from(v.get_source_file()),
                ))
            })
            .collect();

        Ok(Self {
            protected: protected,
        })
    }

    pub fn get_protected(&self) -> &Vec<String> {
        &self.protected
    }
}
