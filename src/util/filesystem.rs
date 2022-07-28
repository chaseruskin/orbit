use fs_extra;
use ignore::WalkBuilder;
use std::ffi::OsStr;
use std::path::{Path, Component};
use home::home_dir;

/// Recursively walks the given `path` and ignores files defined in a .gitignore file.
/// 
/// Returns the resulting list of filepath strings. This function silently skips result errors
/// while walking. The collected set of paths are also standardized to use forward slashes '/'.
/// 
/// Ignores ORBIT_SUM_FILE, .git directory, ORBIT_METADATA_FILE, and IP_LOCK_FILE.
pub fn gather_current_files(path: &std::path::PathBuf) -> Vec<String> {
    let m = WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .filter_entry(|p| {
            match p.file_name().to_str().unwrap() {
                manifest::ORBIT_SUM_FILE | GIT_DIR | lockfile::IP_LOCK_FILE | manifest::ORBIT_METADATA_FILE => false,
                _ => true,
            }
        })
        .build();
    let mut files: Vec<String> = m.filter_map(|result| {
        match result {
            Ok(entry) => {
                if entry.path().is_file() {
                    // replace backslash \ with single forward slash /
                    Some(entry.into_path().display().to_string().replace(r"\", "/"))
                } else {
                    None
                }
            },
            Err(_) => None,
        }
    }).collect();
    // sort the fileset for reproductibility purposes
    files.sort();
    files
}

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

use crate::core::manifest;
use crate::core::lockfile;

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
            normalize_path(resolved_path).display().to_string()
        } else {
            s
        }
    } else {
        s
    }
}

pub fn remove_base(base: &PathBuf, full: &PathBuf) -> PathBuf {
    let mut b_comps = base.iter();
    let mut f_comps = full.iter();
    todo!();
    // while let Some(c) = b_comps.next() {
        
    // }
}

/// Recursively copies files from `source` to `target` directory.
/// 
/// Assumes `target` directory does not already exist. Ignores the `.git/` folder
/// if `ignore_git` is set to `true`.
pub fn copy(source: &PathBuf, target: &PathBuf, ignore_git: bool) -> Result<(), Fault> {
    // create missing directories to `target`
    std::fs::create_dir_all(&target)?;
    // copy contents into cache slot
    let mut from_paths = Vec::new();

    // respect .gitignore
    for result in WalkBuilder::new(&source)
        .hidden(false)
        .filter_entry(move |f| (f.file_name() != GIT_DIR || ignore_git == false))
        .build() {
            match result {
                Ok(entry) => {
                    from_paths.push(entry.path().to_path_buf());
                    // println!("copying: {:?}", entry.path());
                },
                Err(_) => (),
            }
    }

    // create all missing directories
    for from in from_paths.iter().filter(|f| f.is_dir()) {
        // replace common `source` path with `target` path
        let to = PathBuf::from(from.to_str().unwrap().replace(source.to_str().unwrap(), target.to_str().unwrap()));
        std::fs::create_dir_all(&to)?;
    }

    // create all missing files
    for from in from_paths.iter().filter(|f| f.is_file()) {
        // grab the parent
        if let Some(p) = from.parent() {
            let to = PathBuf::from(p.to_str().unwrap().replace(source.to_str().unwrap(), target.to_str().unwrap()))
                .join(from.file_name().unwrap());
            std::fs::copy(from, to)?;
        }
    }

    Ok(())
}

/// This function resolves common filesystem standards into a standardized path format.
/// 
/// It expands leading '~' to be the user's home directory, or expands leading '.' to the
/// current directory. It also handles back-tracking '..' and intermediate current directory '.'
/// notations.
/// 
/// This function is mainly used for display purposes back to the user and is not safe to use
/// for converting filepaths within logic.
pub fn normalize_path(p: std::path::PathBuf) -> PathBuf {
    // break the path into parts
    let mut parts = p.components();

    let c_str = |cmp: Component| {
        match cmp {
            Component::RootDir => String::new(),
            _ => String::from(cmp.as_os_str().to_str().unwrap()),
        }
    };

    let mut result = Vec::<String>::new();
    // check first part for home path '~' and relative path '.'
    if let Some(root) = parts.next() {
        if root.as_os_str() == OsStr::new("~") {
            match home_dir() {
                Some(home) => for part in home.components() { result.push(c_str(part)) }
                None => result.push(String::from(root.as_os_str().to_str().unwrap())),
            }
        } else if root.as_os_str() == OsStr::new(".") {
            for part in std::env::current_dir().unwrap().components() { result.push(c_str(part)) }
        } else {
            result.push(String::from(root.as_os_str().to_str().unwrap()))
        }
    }
    // push remaining components
    while let Some(part) = parts.next() {
        result.push(c_str(part))
    }
    // assemble new path
    let mut first = true;
    PathBuf::from(result.into_iter().fold(String::new(), |x, y| if first == true { first = false; x + &y } else { x + "/" + &y }).replace("\\", "/").replace("//", "/"))
    // @TODO add some fail-safe where if the final path does not exist then return the original path?
}

const GIT_DIR: &str = ".git";

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn resolve_path_simple() {
        let rel_root = std::env::current_dir().unwrap();
        // expands relative path to full path
        assert_eq!(resolve_rel_path(&rel_root, String::from("src/lib.rs")), normalize_path(PathBuf::from("./src/lib.rs")).display().to_string());
        // no file or directory named 'orbit' at the relative root
        assert_eq!(resolve_rel_path(&rel_root, String::from("orbit")), String::from("orbit"));
        // not relative
        assert_eq!(resolve_rel_path(&rel_root, String::from("/src")), String::from("/src"));
    }

    #[test]
    fn normalize() {
        let p = PathBuf::from("~/.orbit/plugins/a.txt");
        assert_eq!(normalize_path(p), PathBuf::from(home_dir().unwrap().join(".orbit/plugins/a.txt").to_str().unwrap().replace("\\", "/")));

        let p = PathBuf::from("home/.././b.txt");
        assert_eq!(normalize_path(p), PathBuf::from("home/../b.txt"));

        let p = PathBuf::from("/home\\c.txt");
        assert_eq!(normalize_path(p), PathBuf::from("/home/c.txt"));

        let p = PathBuf::from("./d.txt");
        assert_eq!(normalize_path(p), PathBuf::from(std::env::current_dir().unwrap().join("d.txt").to_str().unwrap().replace("\\", "/")));
    }

    #[test]
    #[ignore]
    fn rem_base() {
        let base = PathBuf::from("c:/users/kepler/projects");
        let full = PathBuf::from("c:/users/kepler/projects/gates/src/and_gate.vhd");
        assert_eq!(remove_base(&base, &full), PathBuf::from("gates/src/and_gate.vhd"))
    }
}