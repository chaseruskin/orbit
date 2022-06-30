use fs_extra;
use std::path::Path;

pub enum Unit {
    MegaBytes,
    Bytes,
}

impl Unit {
    /// Returns the divisor number to convert to the `self` unit.
    fn value(&self) -> usize {
        match self {
            Self::MegaBytes => 1000000,
            Self::Bytes => 1,
        }
    }
}

/// Calculates the size of the given path.
pub fn compute_size<P>(path: &P, unit: Unit) -> Result<f32, Box<dyn std::error::Error>>
where P: AsRef<Path> {
    Ok(fs_extra::dir::get_size(&path)? as f32 / unit.value() as f32)
}

use std::path::PathBuf;
use std::env;

/// Attempts to return the executable's path.
pub fn get_exe_path() -> Result<PathBuf, Box::<dyn std::error::Error>> {
    match env::current_exe() {    
        Ok(exe_path) => Ok(std::fs::canonicalize(exe_path)?),
        Err(e) => Err(Box::new(e)),
    }
}

/// Resolves a relative path into a full path if given relative to some `root` path.
/// 
/// This function is helpful for resolving full paths in plugin arguments,
/// config.toml includes, and template paths.
pub fn resolve_rel_path(root: &std::path::PathBuf, s: String) -> String {
    let resolved_path = root.join(&s);
    if std::path::Path::exists(&resolved_path) == true {
        if PathBuf::from(&s).is_relative() == true {
            // write out full path
            resolved_path.display().to_string()
        } else {
            s
        }
    } else {
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn resolve_path_simple() {
        let rel_root = std::env::current_dir().unwrap();
        // expands relative path to full path
        assert_eq!(resolve_rel_path(&rel_root, String::from("src/lib.rs")), rel_root.join("src/lib.rs").display().to_string());
        // expands relative path to full path
        assert_eq!(resolve_rel_path(&rel_root, String::from("./src/lib.rs")), rel_root.join("./src/lib.rs").display().to_string());
        // no file or directory named 'orbit' at the relative root
        assert_eq!(resolve_rel_path(&rel_root, String::from("orbit")), String::from("orbit"));
        // not relative
        assert_eq!(resolve_rel_path(&rel_root, String::from("/src")), String::from("/src"));
    }
}