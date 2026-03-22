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

use crate::core::context::CACHE_TAG_FILE;
use crate::core::fileset;
use crate::core::lockfile;
use crate::core::lockfile::PROJECT_LOCK_FILE;
use crate::core::manifest;
use crate::core::manifest::PROJECT_MANIFEST_FILE;
use crate::error::Error;
use crate::info;
use fs_extra;
use home::home_dir;
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::TryLockError;
use std::io::Read;
use std::path::PathBuf;
use std::path::{Component, Path};

use super::anyerror::Fault;

/// Recursively walks the given `path` and ignores files defined in a .gitignore file.
///
/// Returns the resulting list of filepath strings. This function silently skips result errors
/// while walking. The collected set of paths are also standardized to use forward slashes '/'.
///
/// Setting `strip_base` to `true` will remove the overlapping `path` components from the
/// final [String] entries in the resulting vector.
///
/// Ignores ORBIT_SUM_FILE, .git directory, ORBIT_METADATA_FILE, and PROJECT_LOCK_FILE.
///
/// Always ignores folder with a CACHEDIR.TAG file or folders with a Orbit.toml file (besides the
/// current starting directory).
pub fn gather_current_files(path: &PathBuf, strip_base: bool) -> Vec<String> {
    let start_path = path.clone();
    let walker = WalkBuilder::new(&path)
        .hidden(false)
        // NOTE: Whatever filters are applied, be sure to update the `copy` function as well!
        .filter_entry(move |p| {
            // Always ignore folders with the CACHEDIR.TAG file
            if p.path().is_dir() && p.path().join(CACHE_TAG_FILE).exists() == true {
                false
            // Always ignore folders with another Orbit.toml manifest file
            } else if p.path() != start_path.as_path()
                && p.path().is_dir()
                && p.path().join(PROJECT_MANIFEST_FILE).exists() == true
            {
                false
            } else {
                match p.file_name().to_str().unwrap() {
                    manifest::ORBIT_SUM_FILE
                    | lockfile::PROJECT_LOCK_FILE
                    | manifest::ORBIT_CACHE_FILE => false,
                    _ => true,
                }
            }
        })
        .build();
    let mut files: Vec<String> = walker
        .filter_map(|result| {
            match result {
                Ok(entry) => {
                    if entry.path().is_file() {
                        // perform standardization
                        Some(into_std_str(match strip_base {
                            true => remove_base(&path, &entry.into_path()),
                            false => entry.into_path(),
                        }))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .collect();
    // sort the fileset for reproducibility purposes
    files.sort();
    files
}

/// Replaces '\' characters with single '/' character and converts the [PathBuf] into a [String].
pub fn into_std_str(path: PathBuf) -> String {
    let mut s = path.display().to_string().replace(r"\", "/");
    if s.ends_with("/") == true {
        s.pop().unwrap();
    }
    s
}

// NOTE: File locking is here! (rustc 1.89.0) to Rust's standard library, see closed
// issue: https://github.com/rust-lang/rust/issues/130994
//
// In addition, Cargo's codebase has its own filelocking module, see source
// file: https://github.com/rust-lang/cargo/blob/master/src/cargo/util/flock.rs
const LOCK_FILE: &str = ".#lock";

pub const PRJ_CATALOG_EX_LOCK_NAME: &str = "catalog-mutate";
pub const PRJ_CATALOG_SH_LOCK_NAME: &str = "catalog";

#[derive(PartialEq, Debug)]
pub enum LockZone {
    OutputDir,
    ProjectCatalog,
}

/// Attempts to release a lock, returning an error if failed.
pub fn release_lock(fd: &File) -> Result<(), Fault> {
    match fd.unlock() {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

/// Attempts to acquire a lock, blocking until granting access.
///
/// Goes for a shared lock if `shared` is true, otherwise goes for an exclusive
/// lock.
pub fn acquire_lock<P>(
    dir: &P,
    lockzone: LockZone,
    name: Option<&str>,
    shared: bool,
) -> Result<(PathBuf, std::fs::File), Fault>
where
    P: AsRef<Path>,
{
    let name = name.unwrap_or("");
    let lock_path = if name.len() > 0 {
        dir.as_ref().join(&format!("{}-{}", LOCK_FILE, name))
    } else {
        dir.as_ref().join(LOCK_FILE)
    };

    let zone = match lockzone {
        LockZone::OutputDir => "target output directory",
        LockZone::ProjectCatalog => "project catalog directory",
    };

    let mut waiting_on_lock = false;
    let lockfile = loop {
        // verify the directory exists
        if dir.as_ref().exists() == false {
            match std::fs::create_dir_all(&dir) {
                Ok(_) => (),
                Err(e) => return Err(Box::new(e)),
            }
        }
        // create the file if it does not exist
        if lock_path.exists() == false {
            let _ = File::create_new(&lock_path);
        }
        // get the options based on which lock we are trying to access
        let fd_opts = match shared {
            true => std::fs::OpenOptions::new().read(true).to_owned(),
            false => std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .to_owned(),
        };
        if let Ok(fd) = fd_opts.open(&lock_path) {
            // try to secure the lock (shared or unshared)
            let lock_attempt = match shared {
                true => fd.try_lock_shared(),
                false => fd.try_lock(),
            };
            // take action based on locking result
            match lock_attempt {
                Ok(_) => break fd,
                // Lock not acquired
                Err(TryLockError::WouldBlock) => {
                    if waiting_on_lock == false {
                        info!("waiting for file lock on {}", zone);
                    }
                    waiting_on_lock = true;
                }
                Err(TryLockError::Error(err)) => {
                    return Err(Box::new(Error::FileLockFailed(lock_path, err.to_string())))?;
                }
            }
        }
    };
    Ok((lock_path, lockfile))
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
pub fn compute_size<P>(path: &P, unit: Unit) -> Result<f32, Fault>
where
    P: AsRef<Path>,
{
    Ok(fs_extra::dir::get_size(&path)? as f32 / unit.value() as f32)
}

/// Attempts to return the executable's path.
pub fn get_exe_path() -> Result<PathBuf, Fault> {
    match env::current_exe() {
        Ok(exe_path) => Ok(std::fs::canonicalize(exe_path)?),
        Err(e) => Err(Box::new(e)),
    }
}

/// Resolves a relative path into a full path if given relative to some `root` path.
///
/// This function is helpful for resolving full paths in plugin arguments,
/// config.toml includes, and template paths.
pub fn resolve_rel_path(root: &std::path::PathBuf, s: &str) -> String {
    if s.len() == 0 {
        s.to_string()
    } else {
        let resolved_path = root.join(&s);
        if std::path::Path::exists(&resolved_path) == true {
            if PathBuf::from(&s).is_relative() == true {
                // Write out full path.
                PathBuf::standardize(resolved_path).display().to_string()
            } else {
                s.to_string()
            }
        } else {
            s.to_string()
        }
    }
}

/// Resolves a relative path into a full path if given relative to some `root` path.
///
/// This function is helpful for resolving full paths in plugin arguments,
/// config.toml includes, and template paths.
pub fn resolve_rel_path2(root: &std::path::PathBuf, s: &PathBuf) -> PathBuf {
    let resolved_path = root.join(&s);
    if std::path::Path::exists(&resolved_path) == true {
        if PathBuf::from(&s).is_relative() == true {
            // write out full path
            PathBuf::standardize(resolved_path)
        } else {
            s.to_path_buf()
        }
    } else {
        s.to_path_buf()
    }
}

/// Removes common path components from `full` if they are found in `base` on
/// the same iterations.
pub fn remove_base(base: &PathBuf, full: &PathBuf) -> PathBuf {
    let mut b_comps = base.iter();
    let mut f_comps = full.iter();

    let result = loop {
        match f_comps.next() {
            Some(full_c) => match b_comps.next() {
                Some(base_c) => {
                    if full_c == base_c {
                        continue;
                    } else {
                        break PathBuf::from(full_c);
                    }
                }
                None => break PathBuf::from(full_c),
            },
            None => break PathBuf::new(),
        }
    };

    // append remaining components
    result.join(f_comps.as_path())
}

pub fn is_orbit_metadata(s: &str) -> bool {
    s == manifest::PROJECT_MANIFEST_FILE || s == lockfile::PROJECT_LOCK_FILE
}

pub fn is_minimal(name: &str) -> bool {
    fileset::is_hdl(&name) == true || is_orbit_metadata(&name) == true
}

pub fn is_keep_override(target: &PathBuf, vip_list: &HashSet<PathBuf>) -> bool {
    vip_list.contains(target)
}

/// Recursively copies files from `source` to `target` directory.
///
/// Assumes `target` directory does not already exist. Always respects `.gitignore` files.
///
/// If `minimal` is set to true, then ignores hidden files, directories with a CACHEDIR.TAG file,
/// and nested directories with an Orbit.toml file.
pub fn copy(
    source: &PathBuf,
    target: &PathBuf,
    minimal: bool,
    _keep: Option<HashSet<PathBuf>>,
) -> Result<(), Fault> {
    // create missing directories to `target`
    std::fs::create_dir_all(&target)?;
    // gather list of paths to copy
    let mut from_paths = Vec::new();

    let start_path = source.clone();

    let mut walker = WalkBuilder::new(&source);
    walker.hidden(minimal);
    if minimal == true {
        walker.filter_entry(move |f| {
            // Always ignore folder with CACHEDIR.TAG file
            if f.path().is_dir() && f.path().join(CACHE_TAG_FILE).exists() {
                false
            // Always ignore folders with another Orbit.toml manifest file
            } else if f.path() != start_path.as_path()
                && f.path().is_dir()
                && f.path().join(PROJECT_MANIFEST_FILE).exists() == true
            {
                false
            } else {
                true
            }
        });
    }
    // WalkBuilder::new(&source)
    //     .hidden(minimal)
    //     .add_custom_ignore_filename(ORBIT_IGNORE_FILE)
    //     // only capture files that are required by minimal installations
    //     .filter_entry(move |f| {
    //         f.path().is_file() == false
    //             || minimal == false
    //             || is_minimal(&f.file_name().to_string_lossy()) == true
    //             || (keep.is_some()
    //                 && is_keep_override(&f.path().to_path_buf(), &keep.as_ref().unwrap()) == true)
    //     })
    //     .build()
    let walk = walker.build();
    for result in walk {
        match result {
            Ok(entry) => from_paths.push(entry.path().to_path_buf()),
            Err(_) => (),
        }
    }
    // create all missing directories
    for from in from_paths.iter().filter(|f| f.is_dir()) {
        // replace common `source` path with `target` path
        let to = target.join(remove_base(&source, from));
        std::fs::create_dir_all(&to)?;
    }
    // create all missing files
    for from in from_paths.iter().filter(|f| f.is_file()) {
        // grab the parent
        if let Some(parent) = from.parent() {
            let to = target
                .join(remove_base(&source, &parent.to_path_buf()))
                .join(from.file_name().unwrap());
            // println!("copying {:?} -> {:?}", &from, &to);
            std::fs::copy(from, &to)?;
        }
    }
    // remove all empty directories
    for from in from_paths.iter().rev().filter(|f| f.is_dir()) {
        // replace common `source` path with `target` path
        let to = target.join(remove_base(&source, from));
        // check if directory is empty
        if to.read_dir()?.count() == 0 {
            std::fs::remove_dir(to)?;
        }
    }

    // Always include Orbit.toml and Orbit.lock
    let always_move = [PROJECT_MANIFEST_FILE, PROJECT_LOCK_FILE];
    for base_path in always_move {
        let start_source_path = source.join(base_path);
        let end_target_path = target.join(base_path);
        if start_source_path.exists() == true && end_target_path.exists() == false {
            std::fs::copy(start_source_path, end_target_path)?;
        }
    }

    Ok(())
}

/// Recursively copies files from `source` to `target` directory in a "smart" way.
///
/// Assumes `target` directory does not already exist. Always respects `.gitignore` files.
///
/// If `minimal` is set to true, then ignores hidden files, directories with a CACHEDIR.TAG file,
/// and nested directories with an Orbit.toml file.
///
/// Some "smart" behaviors exist:
/// - If source and destination files both exist:
///     - Do nothing if the contents are the same
///     - Write new contents if they differ
/// - If source has a file that destination does not:
///     - Create file and write contents
/// - If destination as a file that source does not:
///     - Remove the file from destination
pub fn smart_copy(
    source: &PathBuf,
    target: &PathBuf,
    minimal: bool,
    _keep: Option<HashSet<PathBuf>>,
) -> Result<(), Fault> {
    // create missing directories to `target`
    if target.exists() == false {
        std::fs::create_dir_all(&target)?;
    }
    // gather list of paths to copy
    let mut from_paths = Vec::new();

    let start_path = source.clone();

    let mut walker = WalkBuilder::new(&source);
    walker.hidden(minimal);
    if minimal == true {
        walker.filter_entry(move |f| {
            // Always ignore folder with CACHEDIR.TAG file
            if f.path().is_dir() && f.path().join(CACHE_TAG_FILE).exists() {
                false
            // Always ignore folders with another Orbit.toml manifest file
            } else if f.path() != start_path.as_path()
                && f.path().is_dir()
                && f.path().join(PROJECT_MANIFEST_FILE).exists() == true
            {
                false
            } else {
                true
            }
        });
    }
    let walk = walker.build();
    for result in walk {
        match result {
            Ok(entry) => from_paths.push(entry.path().to_path_buf()),
            Err(_) => (),
        }
    }

    // create all missing directories
    for from in from_paths.iter().filter(|f| f.is_dir()) {
        // replace common `source` path with `target` path
        let to = target.join(remove_base(&source, from));
        if to.exists() == false {
            std::fs::create_dir_all(&to)?;
        }
    }

    // create all missing files
    for from in from_paths.iter().filter(|f| f.is_file()) {
        // grab the parent
        if let Some(parent) = from.parent() {
            let to = target
                .join(remove_base(&source, &parent.to_path_buf()))
                .join(from.file_name().unwrap());
            // println!("copying {:?} -> {:?}", &from, &to);
            if to.exists() == false {
                std::fs::copy(from, &to)?;
            // perform comparison on existing files that should exist
            } else {
                let from_bytes = {
                    let mut from_fd = std::fs::File::open(&from)?;
                    let mut buf = Vec::new();
                    from_fd.read_to_end(&mut buf)?;
                    buf
                };
                let to_bytes = {
                    let mut to_fd = std::fs::File::open(&to)?;
                    let mut buf = Vec::new();
                    to_fd.read_to_end(&mut buf)?;
                    buf
                };
                // overwrite the contents
                if to_bytes != from_bytes {
                    std::fs::copy(from, &to)?;
                }
            }
        }
    }
    // remove all empty directories
    for from in from_paths.iter().rev().filter(|f| f.is_dir()) {
        // replace common `source` path with `target` path
        let to = target.join(remove_base(&source, from));
        // check if directory is empty
        if to.read_dir()?.count() == 0 {
            std::fs::remove_dir(to)?;
        }
    }

    // gather list of paths that already exist in destination
    let mut to_paths = Vec::new();
    let mut walker = WalkBuilder::new(&target);
    walker.hidden(minimal);
    let walk = walker.build();
    for result in walk {
        match result {
            Ok(entry) => to_paths.push(entry.path().to_path_buf()),
            Err(_) => (),
        }
    }
    // println!("{:#?}", to_paths);
    // remove stale files and directories that exit in destination but not in source
    for to in to_paths.iter().filter(|p| p != &target) {
        let mirrored_src = source.join(remove_base(&target, to));
        if from_paths.contains(&mirrored_src) == false && to.exists() {
            if to.is_file() {
                std::fs::remove_file(to)?;
            } else if to.is_dir() {
                std::fs::remove_dir_all(to)?;
            }
        }
    }

    // Always include Orbit.toml and Orbit.lock
    let always_move = [PROJECT_MANIFEST_FILE, PROJECT_LOCK_FILE];
    for base_path in always_move {
        let start_source_path = source.join(base_path);
        let end_target_path = target.join(base_path);
        if start_source_path.exists() == true && end_target_path.exists() == false {
            std::fs::copy(start_source_path, end_target_path)?;
        }
    }

    Ok(())
}

/// This function creates a universally accepted syntax for a full absolute path.
///
/// Begins with a leading forward slash (`/`) and uses forward slashes as component separators.
/// Removes Prefix components and keeps RootDir components.
///
/// If the path `p` is relative, it will be transformed into its full absolute path.
pub fn to_absolute(p: PathBuf) -> Result<PathBuf, Fault> {
    // ensure the path is in absolute form
    let p = if p.is_relative() == true {
        p.canonicalize()?
    } else {
        p
    };

    // {
    //     let mut p2 = p.components();
    //     while let Some(c) = p2.next() {
    //         println!("{:?}", c);
    //     }
    // }

    Ok(p.components()
        .filter(|f| match f {
            Component::Prefix(_) => false,
            _ => true,
        })
        .collect())
}

// Normalizes the path and resolves any relativity to the current working directory.
pub fn full_normal(p: &PathBuf) -> PathBuf {
    let path = resolve_rel_path(&current_dir().unwrap(), &into_std_str(p.clone()));
    PathBuf::standardize(PathBuf::from(path))
}

pub trait Standardize {
    fn standardize<T>(_: T) -> Self
    where
        T: Into<Self>,
        Self: Sized;
}

impl Standardize for PathBuf {
    /// This function resolves common filesystem standards into a standardized path format.
    ///
    /// It expands leading '~' to be the user's home directory, or expands leading '.' to the
    /// current directory. It also handles back-tracking '..' and intermediate current directory '.'
    /// notations.
    ///
    /// This function is mainly used for display purposes back to the user and is not safe to use
    /// for converting filepaths within logic.
    fn standardize<T>(p: T) -> PathBuf
    where
        T: Into<PathBuf>,
    {
        let p: PathBuf = p.into();
        // break the path into parts
        let mut parts = p.components();

        let c_str = |cmp: Component| match cmp {
            Component::RootDir => String::new(),
            _ => String::from(cmp.as_os_str().to_str().unwrap()),
        };

        let mut result = Vec::<String>::new();
        // check first part for home path '~', absolute path, or other (relative path '.'/None)
        if let Some(root) = parts.next() {
            if root.as_os_str() == OsStr::new("~") {
                match home_dir() {
                    Some(home) => {
                        for part in home.components() {
                            result.push(c_str(part))
                        }
                    }
                    None => result.push(String::from(root.as_os_str().to_str().unwrap())),
                }
            } else if root == Component::RootDir {
                result.push(String::from(root.as_os_str().to_str().unwrap()))
            } else {
                // for part in std::env::current_dir().unwrap().components() { result.push(c_str(part)) }
                match root.as_os_str().to_str().unwrap() {
                    "." => (),
                    ".." => {
                        result.pop();
                        ()
                    }
                    _ => result.push(String::from(root.as_os_str().to_str().unwrap())),
                }
            }
        }
        // push user-defined path (remaining components)
        while let Some(part) = parts.next() {
            match part.as_os_str().to_str().unwrap() {
                // do nothing; remain in the same directory
                "." => (),
                // pop if using a '..'
                ".." => {
                    result.pop();
                    ()
                }
                // push all other components
                _ => result.push(c_str(part)),
            }
        }
        // assemble new path
        let mut first = true;
        PathBuf::from(
            result
                .into_iter()
                .fold(String::new(), |x, y| {
                    if first == true {
                        first = false;
                        x + &y
                    } else {
                        x + "/" + &y
                    }
                })
                .replace("\\", "/")
                .replace("//", "/"),
        )
        // @todo: add some fail-safe where if the final path does not exist then return the original path?
    }
}

/// Executes the process invoking the `cmd` with the following `args`.
///
/// Performs a fix to allow .bat files to be searched on windows given the option
/// is enabled through environment variables.
pub fn invoke(
    cwd: &PathBuf,
    cmd: &String,
    args: &Vec<String>,
    try_again: bool,
    envs: HashMap<&String, &String>,
) -> std::io::Result<std::process::Child> {
    match std::process::Command::new(cmd)
        .current_dir(cwd)
        .args(args)
        .envs(&envs)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(r) => Ok(r),
        Err(e) => {
            // check if there is no file extension
            let repeat = try_again == true
                && match PathBuf::from(cmd).file_name() {
                    Some(fname) => fname.to_string_lossy().contains('.') == false,
                    None => true,
                };
            if repeat == true && e.kind() == std::io::ErrorKind::NotFound {
                invoke(cwd, &format!("{}.bat", cmd), args, false, envs)
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolve_path_simple() {
        // expands relative path to full path
        assert_eq!(
            resolve_rel_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")), "src/lib.rs"),
            into_std_str(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        );
        // no file or directory named 'orbit' at the relative root
        assert_eq!(
            resolve_rel_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")), "orbit"),
            String::from("orbit")
        );
        // not relative
        assert_eq!(
            resolve_rel_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")), "/src"),
            String::from("/src")
        );

        // assert_eq!(resolve_rel_path(&PathBuf::from("D:/a/orbit/orbit/"), "src/lib.rs"), String::from("D:/a/orbit/orbit/src/lib.rs"));
    }

    #[test]
    fn normalize() {
        let p = PathBuf::from("~/.orbit/plugins/a.txt");
        assert_eq!(
            PathBuf::standardize(p),
            PathBuf::from(
                home_dir()
                    .unwrap()
                    .join(".orbit/plugins/a.txt")
                    .to_str()
                    .unwrap()
                    .replace("\\", "/")
            )
        );

        let p = PathBuf::from("home/.././b.txt");
        assert_eq!(PathBuf::standardize(p), PathBuf::from("b.txt"));

        let p = PathBuf::from("/home\\c.txt");
        assert_eq!(PathBuf::standardize(p), PathBuf::from("/home/c.txt"));

        let p = PathBuf::from("./d.txt");
        assert_eq!(PathBuf::standardize(p).display().to_string(), "d.txt");
    }

    #[test]
    fn rem_base() {
        let base = PathBuf::from("c:/users/kepler/projects");
        let full = PathBuf::from("c:/users/kepler/projects/gates/src/and_gate.vhd");
        assert_eq!(
            remove_base(&base, &full),
            PathBuf::from("gates/src/and_gate.vhd")
        );

        let base = PathBuf::from("c:/users/kepler/projects");
        let full = PathBuf::from("c:/users/kepler/projects/");
        assert_eq!(remove_base(&base, &full), PathBuf::new());

        let base = PathBuf::from("");
        let full = PathBuf::from("c:/users/kepler/projects/");
        assert_eq!(
            remove_base(&base, &full),
            PathBuf::from("c:/users/kepler/projects/")
        );

        let base = PathBuf::from("./tests/env/");
        let full = PathBuf::from("./tests/env/Orbit.toml");
        assert_eq!(remove_base(&base, &full), PathBuf::from("Orbit.toml"));
    }

    #[test]
    fn copy_minimal() {
        let source = PathBuf::from("test/data/projects");
        let target = tempdir().unwrap();
        copy(&source, &target.as_ref().to_path_buf(), true, None).unwrap();
    }

    // only works on windows system
    #[test]
    #[ignore]
    fn full_path() {
        let path = PathBuf::from("c://users/cruskin/develop/rust");
        assert_eq!(
            to_absolute(path).unwrap(),
            PathBuf::from("/users/cruskin/develop/rust")
        );

        let path = PathBuf::from("/users/cruskin/develop/rust");
        assert_eq!(
            to_absolute(path).unwrap(),
            PathBuf::from("/Users/cruskin/Develop/rust")
        );

        let path = PathBuf::from("./readme.md");
        assert_eq!(
            to_absolute(path).unwrap(),
            PathBuf::from("/Users/cruskin/Develop/rust/orbit/README.md")
        );
    }
}
