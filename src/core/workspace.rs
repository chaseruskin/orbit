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

use crate::core::lockfile::LockFile;
use crate::core::lockfile::WORKSPACE_LOCK_FILE;
use crate::core::manifest::FromFile;
use crate::core::manifest::ProjectName;
use crate::core::manifest::WORKSPACE_MANIFEST_FILE;
use crate::core::manifest::map_is_empty;
use crate::core::manifest::validate_lib_name;
use crate::error::Error;
use crate::error::LastError;
use crate::util::anyerror::Fault;
use crate::util::filesystem;
use glob::glob;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Workspace {
    root: PathBuf,
    data: WorkspaceManifest,
    lock: LockFile,
}

impl Workspace {
    /// Load a [Workspace] instance from the `root` path.
    ///
    /// If `is_working_prj` is true, then it verifies there are no files created
    /// by the user that are reserved for orbit's internal usage.
    pub fn load(root: PathBuf) -> Result<Self, Fault> {
        let man_path = root.join(WORKSPACE_MANIFEST_FILE);
        if man_path.exists() == false || man_path.is_file() == false {
            return Err(Error::WorkspaceLoadFailed(LastError(
                Error::ManifestPathNotFound(man_path.to_string_lossy().to_string()).to_string(),
            )))?;
        }
        let man = WorkspaceManifest::from_file(&man_path)?;

        let lock_path = root.join(WORKSPACE_LOCK_FILE);

        let lock = match LockFile::from_file(&lock_path) {
            Ok(l) => l,
            Err(e) => {
                crate::warn!(
                    "failed to parse lockfile \"{}\": {}",
                    filesystem::into_std_str(lock_path),
                    e
                );
                LockFile::new()
            }
        };

        Ok(Self {
            root: root,
            data: man,
            lock: lock,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceManifest {
    workspace: WorkspacePackage,
}

impl FromFile for WorkspaceManifest {}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkspacePackage {
    members: Vec<String>,
    #[serde(deserialize_with = "validate_lib_name", default)]
    library: Option<ProjectName>,
    description: Option<String>,
    /// Ignore this field and never use it for any processing.
    #[serde(skip_serializing_if = "map_is_empty", default)]
    metadata: HashMap<String, toml::Value>,
}

impl WorkspaceManifest {
    /// Returns true if the contents at `path` can be parsed into a [WorkspaceManifest].
    pub fn can_parse(path: &PathBuf) -> bool {
        let text = std::fs::read_to_string(&path);
        if text.is_err() {
            return false;
        }
        Self::from_str(&text.unwrap()).is_ok()
    }

    /// Checks if the workspace contains the project directory.
    pub fn has_member(&self, root_dir: &PathBuf, project_dir: &PathBuf) -> bool {
        for member in &self.workspace.members {
            for entry in glob(root_dir.join(member).as_os_str().to_str().unwrap())
                .expect("failed to read workspace manifest's glob pattern")
            {
                if &entry.expect("failed to load workspace's glob result") == project_dir {
                    return true;
                }
            }
        }
        false
    }
}

impl FromStr for WorkspaceManifest {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}
