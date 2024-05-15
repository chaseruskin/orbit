use std::fmt::Display;
use std::path::PathBuf;

pub const ORBIT_PUB_FILE: &str = ".orbitpub";

#[derive(Debug)]
pub struct PubFile<'a> {
    inner: Option<gitignore::File<'a>>,
}

impl<'a> PubFile<'a> {
    pub fn new(path: &'a PathBuf) -> Self {
        match path.exists() {
            true => match gitignore::File::new(path) {
                Ok(r) => Self { inner: Some(r) },
                Err(_) => Self { inner: None },
            },
            false => Self { inner: None },
        }
    }

    pub fn is_included(&self, path: &str) -> bool {
        if let Some(pub_file) = &self.inner {
            // the pub file explicitly lists the files to keep (not excludes)
            match pub_file.is_excluded(&PathBuf::from(path)) {
                Ok(is_inc) => {
                    match is_inc {
                        // keep the file!
                        true => true,
                        // do not keep the file
                        false => false,
                    }
                }
                Err(_) => true,
            }
        } else {
            true
        }
    }

    pub fn exists(root: &PathBuf) -> bool {
        root.join(ORBIT_PUB_FILE).exists()
    }

    pub fn get_filename() -> &'static str {
        ORBIT_PUB_FILE
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
