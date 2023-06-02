//! File     : version.rs
//! Abstract :
//!     A `version` contains numeric values at 3 levels for informing about 
//!     varying degrees of changes within a project's lifetime.
 
use std::num::ParseIntError;
use std::str::FromStr;
use std::error::Error;
use std::fmt::Display;

use crate::util::anyerror::AnyError;

type VerNum = u16;

/// Checks if a partial version `self` umbrellas the full version `ver`.
pub fn is_compatible(pv: &PartialVersion, ver: &Version) -> bool {
    if pv.major != ver.major { return false }

    match pv.minor {
        Some(m) => {
            if m == ver.minor {
                match pv.patch {
                    Some(p) => p == ver.patch,
                    None => true,
                }
            } else {
                false
            }
        }
        None => true,
    }
}

/// Finds the most compatible version matching `ver` among the possible `space`.
/// 
/// Errors if no version was found.
pub fn get_target_version<'a>(ver: &AnyVersion, space: &'a Vec<&Version>) -> Result<Version, AnyError> {
    // find the specified version for the given ip
    let mut latest_version: Option<&Version> = None;
    space.into_iter()
    .filter(|f| match &ver {
        AnyVersion::Specific(v) => crate::core::version::is_compatible(v, f),
        AnyVersion::Latest => true,
    })
    .for_each(|tag| {
        if latest_version.is_none() || *tag > latest_version.as_ref().unwrap() {
            latest_version = Some(tag);
        }
    });
    match latest_version {
        Some(v) => Ok(v.clone()),
        None => Err(AnyError(format!("\
ip has no version available as {}

To see all versions try `orbit probe <ip> --versions`", ver))),
    }
}

#[derive(Debug, Eq, Clone, PartialEq, Ord, PartialOrd)]
pub enum AnyVersion {
    Specific(PartialVersion),
    Latest,
}

impl std::fmt::Display for AnyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => write!(f, "latest"),
            Self::Specific(v) => write!(f, "{}", v),
        }
    }
}

impl std::str::FromStr for AnyVersion {
    type Err = VersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if crate::util::strcmp::cmp_ascii_ignore_case(s, "latest") {
            Ok(Self::Latest)
        } else {    
            Ok(Self::Specific(PartialVersion::from_str(s)?))
        }
    }
}

impl AnyVersion {
    pub fn as_specific(&self) -> Option<&PartialVersion> {
        match self {
            Self::Specific(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_latest(&self) -> bool {
        self == &Self::Latest
    }
}

impl From<&Version> for AnyVersion {
    fn from(value: &Version) -> Self {
        Self::Specific(PartialVersion { 
            major: value.get_major(), 
            minor: Some(value.get_minor()), 
            patch: Some(value.get_patch()),
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Eq, Ord)]
pub struct PartialVersion {
    major: VerNum,
    minor: Option<VerNum>,
    patch: Option<VerNum>,
}

impl PartialVersion {
    pub fn new() -> Self {
        PartialVersion { major: 0, minor: None, patch: None }
    }

    pub fn major(mut self, m: VerNum) -> Self {
        self.major = m;
        self
    }

    pub fn minor(mut self, m: VerNum) -> Self {
        self.minor = Some(m);
        self
    }

    pub fn patch(mut self, m: VerNum) -> Self {
        self.patch = Some(m);
        self
    }

    /// Checks if `other` version is within the same domain as the `self`.
    /// 
    /// This means either version can fulfill the same requirements, where one may
    /// be implicitly higher or stricter.
    pub fn in_domain(&self, other: &PartialVersion) -> bool {
        if self.major != other.major {
            return false
        }
        if self.minor.is_some() && other.minor.is_some() && other.minor.unwrap() != self.minor.unwrap() {
            return false
        }

        other.patch.is_none() || self.patch.is_none() || other.patch.unwrap() == self.patch.unwrap() 
    }

    /// Returns the highest compatible version from the list of versions.
    /// 
    /// Returns `None` if the list is empty or there are zero that meet the
    /// criteria.
    pub fn find_highest<'a>(&self, vers: &'a [Version]) -> Option<&'a Version> {
        let mut highest = None;
        vers.iter().for_each(|v| {
            if is_compatible(self, v) == true && (highest.is_none() || highest.unwrap() < v) {
                highest = Some(v);
            }
        });
        highest
    }

    /// Checks if there are 3 specified version numbers.
    pub fn is_fully_qualified(&self) -> bool {
        self.minor.is_some() && self.patch.is_some()
    }

    /// Returns the partial version as a glob-style pattern.
    pub fn to_pattern_string(&self) -> String {
        format!("{}.{}.{}", 
            self.major, 
            { if self.minor.is_some() { self.minor.unwrap().to_string() } else { "*".to_string() } }, 
            { if self.patch.is_some() { self.patch.unwrap().to_string() } else { "*".to_string() } })
    }
}

impl Display for PartialVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(m) = self.minor {
            write!(f, ".{}", m)?;
            if let Some(p) = self.patch {
                write!(f, ".{}", p)?;
            }
        }
        Ok(())
    }
}

impl From<PartialVersion> for Version {
    fn from(pv: PartialVersion) -> Self { 
        Self { 
            major: pv.major, 
            minor: pv.minor.unwrap_or(0), 
            patch: pv.patch.unwrap_or(0) 
        }
    }
}

impl FromStr for PartialVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        use VersionError::*;

        let s = s.trim();
        if s.is_empty() { return Err(EmptyVersion); }

        let mut levels = s.split_terminator('.')
            .map(|p| { p.parse::<VerNum>() });
        // @TODO handle invalid parses internally to return what level gave invalid digit?
        Ok(PartialVersion {
            major: if let Some(v) = levels.next() {
                v?
            } else {
                return Err(VersionError::MissingMajor);
            }, 
            minor: if let Some(v) = levels.next() {
                Some(v?)
            } else {
                None
            }, 
            patch: if let Some(v) = levels.next() {
                if levels.next().is_some() {
                    return Err(VersionError::ExtraLevels(3+levels.count()));
                }
                Some(v?)
            } else {
                None
            }, 
        })
    }
}

// @TODO make `minor` and `patch` fields optional?

#[derive(Debug, PartialEq, PartialOrd, Clone, Ord, Eq, Hash)]
pub struct Version {
    major: VerNum, 
    minor: VerNum,
    patch: VerNum,
}

use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::de::{self};
use std::fmt;

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Version, D::Error>
        where D: de::Deserializer<'de>
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a semantic version number")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error, {
                
                match Version::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e))
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Version {
    pub fn new() -> Self {
        Version { 
            major: 0, 
            minor: 0, 
            patch: 0,
        }
    }

    /// Increments the `major` level and resets `minor` and `patch` levels.
    pub fn inc_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    /// Increments the `minor` level and resets the `patch` level.
    pub fn inc_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Increments the `patch` level and resets no levels.
    pub fn inc_patch(&mut self) {
        self.patch += 1;
    }

    pub fn major(mut self, m: VerNum) -> Self {
        self.major = m;
        self
    }

    pub fn minor(mut self, m: VerNum) -> Self {
        self.minor = m;
        self
    }

    pub fn patch(mut self, p: VerNum) -> Self {
        self.patch = p;
        self
    }

    pub fn get_major(&self) -> VerNum {
        self.major
    }

    pub fn get_minor(&self) -> VerNum {
        self.minor
    }

    pub fn get_patch(&self) -> VerNum {
        self.patch
    }

    pub fn to_partial_version(&self) -> PartialVersion {
        PartialVersion::new().major(self.major).minor(self.minor).patch(self.patch)
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        use VersionError::*;

        let s = s.trim();
        if s.is_empty() { return Err(EmptyVersion); }

        let mut levels = s.split_terminator('.')
            .map(|p| { p.parse::<VerNum>() });
        // @TODO handle invalid parses internally to return what level gave invalid digit?
        Ok(Version {
            major: if let Some(v) = levels.next() {
                v?
            } else {
                return Err(VersionError::MissingMajor);
            }, 
            minor: if let Some(v) = levels.next() {
                v?
            } else {
                return Err(VersionError::MissingMinor);
            }, 
            patch: if let Some(v) = levels.next() {
                if levels.next().is_some() {
                    return Err(VersionError::ExtraLevels(3+levels.count()));
                }
                v?
            } else {
                return Err(VersionError::MissingPatch);
            }, 
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{}.{}.{}", self.get_major(), self.get_minor(), self.get_patch())
    }
}

#[derive(Debug, PartialEq)]
pub enum VersionError {
    EmptyVersion,
    MissingMajor,
    MissingMinor,
    MissingPatch,
    ExtraLevels(usize),
    InvalidDigit(ParseIntError),
}

impl Error for VersionError {}

impl Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use VersionError::*;
        match self {
            EmptyVersion => write!(f, "empty version"),
            MissingMajor => write!(f, "missing major number"),
            MissingMinor => write!(f, "missing minor number"),
            MissingPatch => write!(f, "missing patch number"),
            ExtraLevels(l) => write!(f, "too many version positions; found {} expected 3", l),
            InvalidDigit(_) => write!(f, "invalid digit in version"),
        }
    }
}

impl From<ParseIntError> for VersionError {
    fn from(e: ParseIntError) -> Self { 
        match e {
            _ => VersionError::InvalidDigit(e)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod partial_ver {
        use super::*;

        #[test]
        fn is_compat() {
            let pv = PartialVersion { major: 1, minor: None, patch: None };
            let v = Version { major: 1, minor: 2, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), true);

            let v = Version { major: 2, minor: 1, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), false);

            let pv = PartialVersion { major: 2, minor: Some(1), patch: None };
            let v = Version { major: 2, minor: 2, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), false);

            let v = Version { major: 2, minor: 1, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), true);

            let v = Version { major: 9, minor: 1, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), false);

            let pv = PartialVersion { major: 2, minor: Some(1), patch: Some(3) };
            let v = Version { major: 2, minor: 1, patch: 3 };
            assert_eq!(is_compatible(&pv, &v), true);
        }

        #[test]
        fn display() {
            let pv = PartialVersion { major: 1, minor: None, patch: None };
            assert_eq!(pv.to_string(), "1");

            let pv = PartialVersion { major: 1, minor: Some(2), patch: None };
            assert_eq!(pv.to_string(), "1.2");

            let pv = PartialVersion { major: 1, minor: Some(2), patch: Some(3) };
            assert_eq!(pv.to_string(), "1.2.3");
        }

        #[test]
        fn find_highest() {
            let pv = PartialVersion { major: 1, minor: None, patch: None };
            let versions = vec![
                Version::new().major(2).minor(1).patch(1),
                Version::new().major(4).minor(2).patch(5),
                Version::new().major(7).minor(9).patch(1),
                Version::new().major(1).minor(2).patch(5),
                Version::new().major(1).minor(3).patch(4),
                Version::new().major(1).minor(0).patch(0),
            ];
            assert_eq!(pv.find_highest(&versions), Some(&Version::new().major(1).minor(3).patch(4)));

            let pv = PartialVersion { major: 4, minor: Some(3), patch: None };
            assert_eq!(pv.find_highest(&versions), None);
        }

        #[test]
        fn from_str() {
            // valid cases
            let v = PartialVersion::from_str("1.2.3").unwrap();
            assert_eq!(v, PartialVersion {
                major: 1, 
                minor: Some(2), 
                patch: Some(3),
            });
            assert_eq!(v.is_fully_qualified(), true);
            let v = PartialVersion::from_str("19.4").unwrap();
            assert_eq!(v, PartialVersion {
                major: 19,
                minor: Some(4),
                patch: None,
            });
            assert_eq!(v.is_fully_qualified(), false);
        }
    }

    #[test]
    fn new() {
        let v: Version = Version::new();
        assert_eq!(v, Version { major: 0, minor: 0, patch: 0 });
        let v = v.major(1).minor(2).patch(3);
        assert_eq!(v, Version { major: 1, minor: 2, patch: 3 });
    }

    #[test]
    fn inc() {
        let mut v = Version { major: 7, minor: 1, patch: 19 };
        v.inc_major();
        assert_eq!(v, Version { major: 8, minor: 0, patch: 0 });

        let mut v = Version { major: 7, minor: 1, patch: 19 };
        v.inc_minor();
        assert_eq!(v, Version { major: 7, minor: 2, patch: 0 });

        let mut v = Version { major: 7, minor: 1, patch: 19 };
        v.inc_patch();
        assert_eq!(v, Version { major: 7, minor: 1, patch: 20 });
    }

    #[test]
    fn from_str() {
        // valid cases
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(v, Version {
            major: 1, 
            minor: 2, 
            patch: 3,
        });
        let v = Version::from_str("19.4.73").unwrap();
        assert_eq!(v, Version {
            major: 19,
            minor: 4,
            patch: 73,
        });
        let v = Version::from_str("1.256.0").unwrap();
        assert_eq!(v, Version {
            major: 1,
            minor: 256,
            patch: 0,
        });
        let v = Version::from_str("019.004.073").unwrap();
        assert_eq!(v, Version {
            major: 19,
            minor: 4,
            patch: 73,
        });
        // invalid cases
        let v = Version::from_str("1.2.");
        assert!(v.is_err());
        let v = Version::from_str("1.abc.7");
        assert!(v.is_err());
        let v = Version::from_str("");
        assert!(v.is_err());
        let v = Version::from_str("1.-4.5");
        assert!(v.is_err());
        let v = Version::from_str("1.4.5.9");
        assert!(v.is_err());
        let v = Version::from_str("1.4.1_5");
        assert!(v.is_err());
    }

    #[test]
    fn cmp() {
        let v0 = Version::new().major(1).minor(2).patch(3);
        let v1  = Version::new().major(1).minor(2).patch(3);
        assert_eq!(v0 == v1, true);
        assert_eq!(v0 != v1, false);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, false);
        assert_eq!(v0 >= v1, true);
        assert_eq!(v0 <= v1, true);

        let v1  = Version::new().major(2).minor(2).patch(3);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1  = Version::new().major(1).minor(3).patch(3);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1  = Version::new().major(1).minor(2).patch(4);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);

        let v1  = Version::new().major(7).minor(80).patch(4);
        assert_eq!(v0 == v1, false);
        assert_eq!(v0 != v1, true);
        assert_eq!(v0 > v1, false);
        assert_eq!(v0 < v1, true);
        assert_eq!(v0 >= v1, false);
        assert_eq!(v0 <= v1, true);
    }

    #[test]
    fn to_str() {
        let v = Version { major: 20, minor: 4, patch: 7 };
        assert_eq!(v.to_string(), "20.4.7");
    }

    #[test]
    fn partial_ver_cmp() {
        let v0 = PartialVersion::new().major(1);
        let v1 = PartialVersion::new().major(1).minor(5);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);

        let v0 = PartialVersion::new().major(1).minor(1);
        let v1 = PartialVersion::new().major(1).minor(0);
        assert_eq!(v0.in_domain(&v1), false);
        assert_eq!(v1.in_domain(&v0), false);

        let v0 = PartialVersion::new().major(2).minor(1).patch(3);
        let v1 = PartialVersion::new().major(1).minor(0).patch(4);
        assert_eq!(v0.in_domain(&v1), false);
        assert_eq!(v1.in_domain(&v0), false);

        let v0 = PartialVersion::new().major(1).minor(0).patch(4);
        let v1 = PartialVersion::new().major(1).minor(0).patch(4);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);

        let v0 = PartialVersion::new().major(1);
        let v1 = PartialVersion::new().major(1).minor(2).patch(4);
        assert_eq!(v0.in_domain(&v1), true);
        assert_eq!(v1.in_domain(&v0), true);
    }
}