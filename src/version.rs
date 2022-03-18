//! File     : version.rs
//! Abstract :
//!     A `version` contains numeric values at 3 levels for informing about 
//!     varying degrees of changes within a project's lifetime.
 
use std::num::ParseIntError;
use std::str::FromStr;
use std::error::Error;
use std::fmt::Display;

type VerNum = u16;

// :todo: make `minor` and `patch` fields optional?

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version {
    major: VerNum, 
    minor: VerNum,
    patch: VerNum,
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
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        use VersionError::*;

        let s = s.trim();
        if s.is_empty() { return Err(EmptyVersion); }

        let mut levels = s.split_terminator('.')
            .map(|p| { p.parse::<VerNum>() });
        // :todo: handle invalid parses internally to return what level gave invalid digit?
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
            MissingMajor => write!(f, "missing major level"),
            MissingMinor => write!(f, "missing minor level"),
            MissingPatch => write!(f, "missing patch level"),
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
}