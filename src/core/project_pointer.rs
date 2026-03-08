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

use super::{
    manifest::{self, Manifest, PROJECT_MANIFEST_FILE},
    project::Project,
};
use crate::error::Error;
use crate::error::LastError;
use crate::util::anyerror::Fault;
use std::path::PathBuf;

/// The project pointer stores the manifest for a project, to be used to grab the project from another
/// location not already on the user's local file system.
#[derive(Debug, PartialEq)]
pub struct ProjectPointer {
    manifest: Manifest,
}

impl ProjectPointer {
    pub fn decouple(self) -> Manifest {
        self.manifest
    }

    /// Loads an [ProjectPointer] struct.
    pub fn read(path: PathBuf) -> Result<Self, Fault> {
        let man_path = path.join(PROJECT_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(Error::ProjectLoadFailed(LastError(
                Error::ManifestPathNotFound(man_path.to_string_lossy().to_string()).to_string(),
            )))?;
        }
        let man = Manifest::from_file(&man_path, false)?;
        Ok(Self { manifest: man })
    }

    /// Finds all Manifest files available in the provided path `path`.
    ///
    /// Errors if on filesystem problems.
    pub fn detect_all(path: &PathBuf) -> Result<Vec<Project>, Fault> {
        let mut result = Vec::new();
        // walk the directory
        for mut entry in manifest::find_file(&path, PROJECT_MANIFEST_FILE, false)? {
            // remove the manifest file to access the project's root directory
            entry.pop();
            result.push({
                let ptr = ProjectPointer::read(entry)?;
                Project::from(ptr)
            });
        }
        Ok(result)
    }
}
