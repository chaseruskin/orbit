use std::path::PathBuf;
use std::str::FromStr;
use git2::build::CheckoutBuilder;
use git2::Repository;

use crate::util::anyerror::Fault;

use super::version::Version;

/// A series of git commands necessary to run through subprocesses rather than libgit2 bindings.
pub struct ExtGit {
    command: String,
    root: std::path::PathBuf,
}

impl ExtGit {
    /// Creates an empty `ExtGit` struct.
    /// 
    /// By default, if `cmd` is `None` then `self.command` is set to "git".
    pub fn new(cmd: Option<&str>) -> Self {
        Self {
            command: cmd.unwrap_or("git").to_string(),
            root: PathBuf::new(),
        }
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
    /// 
    /// The `disable_ssh` parameter will convert a url to HTTPS if given as SSH.
    pub fn clone(&self, url: &crate::util::url::Url, dest: &std::path::PathBuf, disable_ssh: bool) -> Result<(), Fault> {
        let tmp_path = tempfile::tempdir()?;
        // check if to convert to https when disabling ssh
        let url = match disable_ssh {
            true => url.as_https().to_string(),
            false => url.to_string()
        };
        let proc = std::process::Command::new(&self.command)
            .args(["clone", &url])
            .current_dir(&tmp_path)
            .output()?;

        match proc.status.code() {
            Some(num) => if num != 0 { Err(ExtGitError::NonZeroCode(num, proc.stderr))? } else { () },
            None => return Err(ExtGitError::SigTermination)?,
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
    pub fn remote_update(&self) -> Result<(), Fault> {
        let output = std::process::Command::new(&self.command)
            .args(["remote", "update"])
            .current_dir(&self.root)
            .output()?;
        match output.status.code() {
            Some(num) => if num != 0 { Err(ExtGitError::NonZeroCode(num, output.stderr))? } else { () },
            None => return Err(ExtGitError::SigTermination)?,
        };
        Ok(())
    }

    /// Pushes to remote repository at `path`.
    /// 
    /// Runs the command: `git push` and `git push --tags`.
    pub fn push(&self) -> Result<(), Fault> {
        let output = std::process::Command::new(&self.command)
            .args(["push"])
            .current_dir(&self.root)
            .output()?; // hide output from reaching stdout by using .output()
        match output.status.code() {
            Some(num) => if num != 0 { Err(ExtGitError::NonZeroCode(num, output.stderr))? } else { () },
            None => return Err(ExtGitError::SigTermination)?,
        };
        // push tags
        let output = std::process::Command::new(&self.command)
            .args(["push", "--tags"])
            .current_dir(&self.root)
            .output()?;
        match output.status.code() {
            Some(num) => if num != 0 { Err(ExtGitError::NonZeroCode(num, output.stderr))? } else { () },
            None => return Err(ExtGitError::SigTermination)?,
        };
        Ok(())
    }

    /// Takes a repository `repo` and forces the checkout to be at the `tag` commit.
    pub fn checkout_tag_state(repo: &Repository, tag: &Version) -> Result<(), Fault> {
        // get the tag
        let obj = repo.revparse_single(tag.to_string().as_ref())?;
        // configure checkout options
        let mut cb = CheckoutBuilder::new();
        cb.force();
        // checkout code at the tag's marked timestamp
        Ok(repo.checkout_tree(&obj, Some(&mut cb))?)
    }

    /// Collects all version git tags from the given `repo` repository.
    /// 
    /// The tags must follow semver `[0-9]*.[0-9]*.[0-9]*` specification.
    pub fn gather_version_tags(repo: &Repository) -> Result<Vec<Version>, Fault> {
        let tags = repo.tag_names(Some("*.*.*"))?;
        Ok(tags.into_iter()
            .filter_map(|f| {
                match Version::from_str(f?) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            })
            .collect())
    }

    /// Returns the highest tagged version known in the given `repo`.
    /// 
    /// This `fn` can be used on a storage dir to detect if an update/pull/replacement 
    /// needs to be performed.
    pub fn get_highest_tag(repo: &Repository) -> Result<Option<Version>, Fault> {
        let mut result: Option<Version> = None;
        Self::gather_version_tags(repo)?.into_iter().for_each(|v| {
            if result.is_none() || &v > result.as_ref().unwrap() {
                result = Some(v);
            }
        });
        Ok(result)
    }

    /// Forces a checkout to be at the tip HEAD.
    pub fn checkout_head(repo: &Repository) -> Result<(), Fault> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        // configure checkout options
        let mut cb = CheckoutBuilder::new();
        cb.force();
        // checkout code at the tag's marked timestamp
        Ok(repo.checkout_tree(&obj, Some(&mut cb))?)
    }

    /// Retrieves the latest commit in the current repository using git2 API.
    /// 
    /// Source: https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html
    pub fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit().map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
}

#[derive(Debug)]
enum ExtGitError {
    NonZeroCode(i32, Vec<u8>),
    SigTermination,
}

impl std::error::Error for ExtGitError {}

impl std::fmt::Display for ExtGitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonZeroCode(num, reason) => write!(f, "exited with error code: {} due to {}", num, String::from_utf8_lossy(reason)),
            Self::SigTermination => write!(f, "terminated by signal"),
        }
    }
}