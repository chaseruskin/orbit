use std::path::PathBuf;

const ORBIT_PUB_FILE: &str = ".orbitpub";

#[derive(Debug)]
pub struct PubFile<'a> {
    inner: gitignore::File<'a>,
}

impl<'a> PubFile<'a> {
    pub fn new(path: &'a PathBuf) -> Result<Self, gitignore::Error> {
        Ok(Self {
            inner: gitignore::File::new(path)?,
        })
    }

    pub fn is_included(&self, path: &str) -> bool {
        // the pub file explicitly lists the files to keep (not excludes)
        match self.inner.is_excluded(&PathBuf::from(path)) {
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
    }

    pub fn exists(root: &PathBuf) -> bool {
        root.join(ORBIT_PUB_FILE).exists()
    }

    pub fn get_filename() -> &'static str {
        ORBIT_PUB_FILE
    }
}
