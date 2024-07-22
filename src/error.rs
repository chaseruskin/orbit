//
//  Copyright (C) 2022-2024  Chase Ruskin
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

use colored::Colorize;
use std::{fmt::Display, path::PathBuf};

use crate::core::{
    blueprint::Scheme,
    ip::IpSpec,
    lang::LangIdentifier,
    pkgid::PkgPart,
    version::{PartialVersion, Version},
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
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
    #[error("duplicate identifier \"{0}\" found in the following source files:\n\n  location 1: {1:?}\n  location 2: {2:?}{3}")]
    DuplicateIdentifiersCrossLang(String, String, String, Hint),
    #[error(
        "blueprint plan \"{0}\" not supported by the current target; supported plans are: {1:?}"
    )]
    BlueprintPlanNotSupported(Scheme, Vec<Scheme>),
    #[error("blueprint plan \"{0}\" not supported by the current target; no plans are defined so it can only accept \"{1}\"")]
    BlueprintPlanMustBeDefault(Scheme, Scheme),
    #[error("failed to find unit with matching name \"{0}\"{1}")]
    GetUnitNotFound(String, Hint),
    #[error("unit \"{0}\" is not a usable design component{1}")]
    GetUnitNotComponent(String, Hint),
    #[error("failed to load ip: {0}")]
    IpLoadFailed(LastError),
    #[error("failed to parse ip name: {0}")]
    IpNameParseFailed(LastError),
    #[error("listed version {0} does not match ip's actual version {1}")]
    DependencyIpRelativeBadVersion(PartialVersion, Version),
    #[error("listed name {0} does not match ip's actual name {1}")]
    DependencyIpRelativeBadName(PkgPart, PkgPart),
    #[error("failed to load lockfile: {0}")]
    LockfileLoadFailed(LastError),
    #[error("failed to install: {0}")]
    InstallFailed(LastError),
    #[error("ip has dependencies that are relative")]
    IpHasRelativeDependencies,
    #[error("a testbench is required to test")]
    TestbenchRequired,
    #[error("top \"{0}\" is not tested in testbench \"{1}\"{2}")]
    TopNotInTestbench(LangIdentifier, LangIdentifier, Hint),
    #[error("lockfile entry \"{0}\" is not queued for installation (missing download)")]
    EntryMissingDownload(IpSpec),
    #[error("lockfile entry \"{0}\" is not queued for installation")]
    EntryNotQueued(IpSpec),
    #[error("lockfile entry \"{0}\" is not queued for installation (unknown ip)")]
    EntryUnknownIp(IpSpec),
    #[error("cannot disambiguate between {0} ips downloaded{1}")]
    DownloadFoundManyIps(usize, Hint),
    #[error("lockfile is missing or out of date{0}")]
    PublishMissingLockfile(Hint),
    #[error("the ip manifest's source field is required to publish, but is undefined")]
    PublishMissingSource,
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
    ShowAvailableUnitsLocal,
    ShowAvailableUnitsExternal(IpSpec),
    DutSpecify,
    WantsTestbench,
    WantsTop,
    TopSpecify,
    BenchSpecify,
    RootSpecify,
    IncludeAllInPlan,
    SpecifyIpSpecForDownload,
    MakeLock,
}

impl Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mixed_prompt = match self {
            Self::ShowAvailableUnitsExternal(spec) => Some(format!(
                "use `orbit view {0} --units` to display available units",
                spec
            )),
            _ => None,
        };
        let message = match self {
            Self::CatalogList => "use `orbit search` to see the list of known ips",
            Self::TargetsList => "use `orbit build --list` to see the list of defined targets",
            Self::InitNotNew => "use `orbit init` to initialize an existing directory",
            Self::IpNameSeparate => {
                "use the \"--name\" option for making an ip name separate from the directory name"
            }
            Self::ResolveDuplicateIds1 => HINT_1,
            Self::ResolveDuplicateIds2 => HINT_2,
            Self::ShowAvailableUnitsLocal => "use `orbit view --units` to display available units",
            Self::ShowAvailableUnitsExternal(_) => mixed_prompt.as_ref().unwrap(),
            Self::DutSpecify => "use the \"--dut\" option to specify the design under test",
            Self::WantsTestbench => {
                "use `orbit test` and its \"--tb\" option to select testbenches"
            }
            Self::WantsTop => {
                "use `orbit build` and its \"--top\" option to select top-level designs"
            }
            Self::TopSpecify => "use the \"--top\" option to specify the top-level design",
            Self::BenchSpecify => "use the \"--tb\" option to specify the testbench",
            Self::RootSpecify => "use the \"--root\" option to specify the root design unit",
            Self::IncludeAllInPlan => "use the \"-all\" flag to continue with this setup",
            Self::SpecifyIpSpecForDownload => {
                "consider providing the ip specification for the requested ip to download"
            }
            Self::MakeLock => "use `orbit lock` to generate the latest lockfile for this ip",
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
