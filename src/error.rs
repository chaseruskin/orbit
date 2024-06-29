use colored::Colorize;
use std::{fmt::Display, path::PathBuf};

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
    #[error("a target must be defined")]
    MissingRequiredTarget,
    #[error("command must be ran from the current working ip: no ip found in current directory or any parent directory")]
    NoWorkingIpFound,
    #[error("command must be ran from the current working ip when an ip is not explicitly defined: no ip found in current directory or any parent directory")]
    NoAssumedWorkingIpFound,
    #[error("ip {0:?} does not exist in the cache")]
    IpNotFoundInCache(String),
    #[error("ip {0:?} does not exist in the catalog{1}")]
    IpNotFoundAnywhere(String, Hint),
    #[error("exited with error code: {0}")]
    ChildProcErrorCode(i32),
    #[error("terminated by signal")]
    ChildProcTerminated,
    #[error("no target named {0:?}{1}")]
    TargetNotFound(String, Hint),
    #[error("a target must be specified{0}")]
    TargetNotSpecified(Hint),
    #[error("failed to execute target process: {0}")]
    TargetProcFailed(LastError),
    #[error("failed to execute protocol process: {0}")]
    ProtocolProcFailed(LastError),
    #[error("no protocol named {0:?}")]
    ProtocolNotFound(String),
    #[error("failed to modify configuration: {0}")]
    ConfigNotSaved(LastError),
    #[error("configuration field {0:?} does not store a list")]
    ConfigFieldNotList(String),
    #[error("failed to process value {1:?} for configuration field \"include\" at {0:?}: {2}")]
    ConfigIncludeFailed(PathBuf, PathBuf, LastError),
    #[error("failed to load configuration file at {0:?}: {1}")]
    ConfigLoadFailed(PathBuf, LastError),
    #[error("failed to parse source code file {0:?}: {1}")]
    SourceCodeInvalidSyntax(PathBuf, LastError),
    #[error("failed to process ip graph: {0}")]
    IpGraphFailed(LastError),
    #[error("failed to parse identifier: {0}")]
    CrossIdentifierParsingFailed(LastError),
}

#[derive(Debug, PartialEq)]
pub struct LastError(pub String);

impl Display for LastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Error::lowerize(self.0.to_string()))
    }
}

impl Error {
    pub fn lowerize(s: String) -> String {
        // get the first word
        let first_word = s.split_whitespace().into_iter().next().unwrap();
        // retain punctuation if the first word is all-caps and longer than 1 character
        if first_word.len() > 1
            && first_word
                .chars()
                .find(|c| c.is_ascii_lowercase() == true)
                .is_none()
        {
            s.to_string()
        } else {
            s.char_indices()
                .map(|(i, c)| if i == 0 { c.to_ascii_lowercase() } else { c })
                .collect()
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Hint {
    TargetsList,
    CatalogList,
    InitNotNew,
    IpNameSeparate,
    ResolveDuplicateIds1,
    ResolveDuplicateIds2,
}

impl Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::CatalogList => "use `orbit search` to see the list of known ips",
            Self::TargetsList => "use `orbit build --list` to see the list of defined targets",
            Self::InitNotNew => "use `orbit init` to initialize an existing directory",
            Self::IpNameSeparate => {
                "see the \"--name\" flag for making an ip name separate from the directory name"
            }
            Self::ResolveDuplicateIds1 => HINT_1,
            Self::ResolveDuplicateIds2 => HINT_2,
        };
        write!(
            f,
            "\n\n{}: {}",
            "hint".green(),
            Error::lowerize(message.to_string())
        )
    }
}

const HINT_1: &str = "resolve this error by either
    1) renaming one of the units to a unique identifier
    2) adding one of the file paths to a .orbitignore file";

const HINT_2: &str = "resolve this error by either
    1) renaming the unit in the working ip to a unique identifier
    2) removing the direct dependency from Orbit.toml
    3) adding the file path for the working ip's unit to a .orbitignore file";
