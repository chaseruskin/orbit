use std::path::PathBuf;
use std::{error::Error, fmt::Display};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

#[derive(Debug)]
pub struct PublicList {
    inner: Option<Gitignore>,
}

impl Default for PublicList {
    fn default() -> Self {
        Self { inner: None }
    }
}

impl PublicList {
    pub fn new(root: &PathBuf, list: &Option<Vec<String>>) -> Result<Self, Box<dyn Error>> {
        let plist = match list {
            Some(globs) => {
                let mut builder = GitignoreBuilder::new(&root);
                for g in globs {
                    builder.add_line(None, g)?;
                }
                Some(builder.build()?)
            }
            None => None,
        };

        Ok(Self { inner: plist })
    }

    /// Checks if the given filepath is included. If there is no public list,
    /// then it will always return true.
    pub fn is_included(&self, path: &str) -> bool {
        match &self.inner {
            Some(ig) => ig.matched_path_or_any_parents(path, false).is_ignore(),
            None => true,
        }
    }

    pub fn exists(&self) -> bool {
        self.inner.is_some()
    }
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Visibility {
    pub fn new() -> Self {
        Self::Public
    }

    pub fn is_public(&self) -> bool {
        self == &Self::Public
    }

    pub fn is_protected(&self) -> bool {
        self == &Self::Protected
    }

    pub fn is_private(&self) -> bool {
        self == &Self::Private
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Public => "public",
                Self::Protected => "protected",
                Self::Private => "private",
            }
        )
    }
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Public
    }
}
