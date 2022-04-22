use toml_edit::Document;
use std::path;
use std::error::Error;

pub struct Manifest {
    // track where the file loads/stores from
    path: path::PathBuf, 
    // maintain the data
    document: Document
}

impl Manifest {
    pub fn new(path: path::PathBuf) -> Self {
        Self {
            path: path,
            document: BARE_MANIFEST.parse::<Document>().unwrap(),
        }
    }

    pub fn get_mut_doc(&mut self) -> &mut Document {
        &mut self.document
    }

    /// Loads data from file as a `Manifest` struct.
    pub fn load(path: path::PathBuf) -> Result<Self, Box<dyn Error>>{
        // load data from file
        let contents = std::fs::read_to_string(&path)?;
        Ok(Self {
            path: path,
            document: contents.parse::<Document>()?,
        })
    }

    /// Stores data to file from `Manifest` struct.
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        std::fs::write(&self.path, self.document.to_string())?;
        Ok(())
    }
}

const BARE_MANIFEST: &str = "\
[ip]
name    = \"\"
library = \"\"
version = \"0.1.0\"
vendor  = \"\"

# To learn more about writing the manifest, see https://github.com/c-rus/orbit

[dependencies]
";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let m = tempfile::NamedTempFile::new().unwrap();
        let manifest = Manifest::new(m.path().to_path_buf());
        assert_eq!(manifest.document.to_string(), BARE_MANIFEST);
    }
}