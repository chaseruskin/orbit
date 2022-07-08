use fs_extra;
use std::ffi::OsStr;
use std::path::{Path, Component};
use home::home_dir;

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

use super::anyerror::Fault;

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

/// This function resolves common filesystem standards into a standardized path format.
/// 
/// It expands leading '~' to be the user's home directory, or expands leading '.' to the
/// current directory. It also handles back-tracking '..' and intermediate current directory '.'
/// notations.
pub fn normalize_path(p: std::path::PathBuf) -> std::path::PathBuf {
    // break the path into parts
    let mut parts = p.components();

    let mut result = Vec::<String>::new();
    // check first part for home path '~' and relative path '.'
    if let Some(root) = parts.next() {
        if root.as_os_str() == OsStr::new("~") {
            match home_dir() {
                Some(home) => for c in home.components() { 
                    match c {
                        Component::RootDir => (),
                        _ => result.push(String::from(c.as_os_str().to_str().unwrap())),
                    }
                },
                None => result.push(String::from(root.as_os_str().to_str().unwrap())),
            }
        } else if root.as_os_str() == OsStr::new(".") {
            for c in std::env::current_dir().unwrap().components() {
                match c {
                    Component::RootDir => (),
                    _ => result.push(String::from(c.as_os_str().to_str().unwrap())),
                }
            }
        } else if root.as_os_str() == OsStr::new("..") {
            for c in std::env::current_dir().unwrap().components() {
                match c {
                    Component::RootDir => (),
                    _ => result.push(String::from(c.as_os_str().to_str().unwrap())),
                }
            }
            result.pop();
        } else {
            result.push(String::from(root.as_os_str().to_str().unwrap()))
        }
    }
    // push remaining components
    while let Some(part) = parts.next() {
        match part {
            Component::RootDir => (),
            _ => {
                if part.as_os_str() == OsStr::new("..") {
                    result.pop();
                } else if part.as_os_str() != OsStr::new(".") {
                    result.push(String::from(part.as_os_str().to_str().unwrap()));
                }
            }
        }
    }
    // assemble new path
    let mut first = true;
    let normal_path = if p.starts_with("/") || p.starts_with("\\") { 
        String::from("/")
    } else {
        String::new()
    };
    PathBuf::from(result.into_iter().fold(normal_path, |x, y| if first == true { first = false; x + &y } else { x + "/" + &y }))
    // @TODO add some fail-safe where if the final path does not exist then return the original path?
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

    #[test]
    fn normalize() {
        let p = PathBuf::from("~/.orbit/plugins/a.txt");
        assert_eq!(normalize_path(p), PathBuf::from(home_dir().unwrap().join(".orbit/plugins/a.txt").to_str().unwrap().replace("\\", "/")));

        let p = PathBuf::from("/home/.././b.txt");
        assert_eq!(normalize_path(p), PathBuf::from("/b.txt"));

        let p = PathBuf::from("/home\\c.txt");
        assert_eq!(normalize_path(p), PathBuf::from("/home/c.txt"));

        let p = PathBuf::from("./b.txt");
        assert_eq!(normalize_path(p), PathBuf::from(std::env::current_dir().unwrap().join("b.txt").to_str().unwrap().replace("\\", "/")));
    }
}