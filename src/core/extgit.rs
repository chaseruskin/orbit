use std::path::PathBuf;

use crate::util::anyerror::AnyError;

/// A series of git commands necessary to run through subprocesses rather than libgit2 bindings.
pub struct ExtGit {
    command: String,
    root: std::path::PathBuf,
}

impl ExtGit {
    /// Creates an empty `ExtGit` struct.
    pub fn new() -> Self {
        Self {
            command: String::new(),
            root: PathBuf::new(),
        }
    }

    /// Sets the command for calling git through processes.
    /// 
    /// By `s` is `None`, the command assumes git is on path and is simply `git`.
    pub fn command(mut self, s: Option<String>) -> Self {
        self.command = s.unwrap_or("git".to_string());
        self
    }

    /// Sets the directory from where to call `git`.
    pub fn path(mut self, p: std::path::PathBuf) -> Self {
        self.root = p;
        self
    }

    /// Clones a repository `url` to `dest`.
    /// 
    /// This function uses the actual git command in order to bypass a lot of issues with using libgit with
    /// private repositories.
    pub fn clone(&self, url: &str, dest: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let tmp_path = tempfile::tempdir()?;

        let mut proc = std::process::Command::new(&self.command).args(["clone", url]).current_dir(&tmp_path).spawn()?;
        let exit_code = proc.wait()?;
        match exit_code.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { () },
            None => return Err(AnyError(format!("terminated by signal")))?,
        };
        // create the directories
        std::fs::create_dir_all(&dest)?;

        // there should only be one directory in the tmp/ folder
        for entry in std::fs::read_dir(&tmp_path)? {
            // copy contents into cache slot
            let temp = entry.unwrap().path();
            let options = fs_extra::dir::CopyOptions::new();
            let mut from_paths = Vec::new();
            for dir_entry in std::fs::read_dir(temp)? {
                match dir_entry {
                    Ok(d) => from_paths.push(d.path()),
                    Err(_) => (),
                }
            }
            // copy rather than rename because of windows issues
            fs_extra::copy_items(&from_paths, &dest, &options)?;
            break;
        }
        Ok(())
    }

    /// Updates a remote repository is up-to-date at `self.root`.
    /// 
    /// Runs the command: `git remote update`.
    pub fn remote_update(&self) -> Result<(), Box<dyn std::error::Error>> {
        let status = std::process::Command::new(&self.command).args(["remote", "update"]).current_dir(&self.root).status()?;
        match status.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { () },
            None => return Err(AnyError(format!("terminated by signal")))?,
        };
        Ok(())
    }

    /// Pushes to remote repository at `path`.
    /// 
    /// Runs the command: `git push` and `git push --tags`.
    pub fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        let status = std::process::Command::new(&self.command).args(["push"]).current_dir(&self.root).status()?;
        match status.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { () },
            None => return Err(AnyError(format!("terminated by signal")))?,
        };
        // push tags
        let status = std::process::Command::new(&self.command).args(["push", "--tags"]).current_dir(&self.root).status()?;
        match status.code() {
            Some(num) => if num != 0 { Err(AnyError(format!("exited with error code: {}", num)))? } else { () },
            None => return Err(AnyError(format!("terminated by signal")))?,
        };
        Ok(())
    }
}