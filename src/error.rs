use colored::Colorize;
use std::path::PathBuf;

type LastError = String;
type Hint = String;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("an ip already exists at {0:?}")]
    IpExistsAtPath(PathBuf),
    #[error("path {0:?} already exists {1}")]
    PathAlreadyExists(PathBuf, Hint),
    #[error("directory {0:?} is an invalid ip name: {1}{2}")]
    CannotAutoExtractNameFromPath(String, LastError, Hint),
    #[error("file system path {0:?} is missing a name{1}")]
    MissingFileSystemPathName(PathBuf, Hint),
    #[error("failed to create new ip: {0}")]
    FailedToCreateNewIp(LastError),
    #[error("failed to initialize ip: {0}")]
    FailedToInitIp(LastError),
}

impl Error {
    pub fn hint(s: &str) -> String {
        format!("\n\n{}: {}", "hint".green(), Self::lowerize(s.to_string()))
    }

    pub fn lowerize(s: String) -> String {
        s.char_indices()
            .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
            .collect()
    }
}
