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
