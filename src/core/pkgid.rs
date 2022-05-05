//! File     : pkgid.rs
//! Abstract :
//!     A `pkgid` is formed is a unique string following VLNV format that allows
//!     reference to a particular package/ip.

use std::str::FromStr;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct PkgPart(String);

impl PkgPart {
    pub fn new() -> Self {
        PkgPart(String::new())
    }
}

impl AsRef<std::path::Path> for PkgPart {
    fn as_ref(&self) -> &std::path::Path {
        self.0.as_ref()
    }
}

impl From<&PkgPart> for toml_edit::Value {
    fn from(p: &PkgPart) -> Self {
        From::<&String>::from(&p.0)
    }
}

impl std::str::FromStr for PkgPart {
    type Err = PkgIdError;

    /// Verifies a part follows the `PkgId` specification.
    /// 
    /// First character must be `alphabetic`. Remaining characters must be
    /// `ascii alphanumeric`, `-`, or `_`.
    fn from_str(s: &str) -> Result<Self, PkgIdError> {
        use PkgIdError::*;

        if let Some(c) = s.chars().next() {
            if c.is_ascii_alphabetic() == false {
                return Err(NotAlphabeticFirst(s.to_owned()));
            }
        }
        // find first char in pkgid part not following spec
        let result = s.chars()
            .find(|&c| {
                !c.is_ascii_alphanumeric() && !(c == '_') && !(c == '-')
            }
        );
        if let Some(r) = result {
            Err(InvalidChar(s.to_owned(), r))
        } else {
            Ok(PkgPart(s.to_owned()))
        }
    }
}

impl std::cmp::PartialEq for PkgPart {
    fn eq(&self, other: &Self) -> bool {
        self.0.replace('-', "_").to_lowercase() == other.0.replace('-', "_").to_lowercase()
    }

    fn ne(&self, other: &Self) -> bool {
        self.eq(other) == false
    }
}

impl std::fmt::Display for PkgPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct PkgId {
    vendor: Option<PkgPart>,
    library: Option<PkgPart>,
    name: PkgPart
}

impl PkgId {
    pub fn new() -> Self {
        PkgId {
            vendor: None,
            library: None,
            name: PkgPart::new(),
        }
    }

    pub fn name(mut self, n: &str) -> Result<Self, PkgIdError> {
        self.name = PkgPart::from_str(n)?;
        Ok(self)
    }

    pub fn library(mut self, l: &str) -> Result<Self, PkgIdError> {
        self.library = Some(PkgPart::from_str(l)?);
        Ok(self)
    }

    pub fn vendor(mut self, v: &str) -> Result<Self, PkgIdError> {
        self.vendor = Some(PkgPart::from_str(v)?);
        Ok(self)
    }

    /// Two `PkgId`'s are considered equivalent if they have identical case 
    /// insensitive string parts. Different than `==` operator. Converting '-' 
    /// to '_' is also applied.
    pub fn equivalent(&self, other: &Self) -> bool {
        self.name == other.name && self.library == other.library && self.vendor == other.vendor
    }

    /// Checks if all the parts for a `PkgId` are specified and nonempty.
    pub fn fully_qualified(&self) -> Result<(), PkgIdError> {
        if self.name.0.len() == 0 {
            Err(PkgIdError::Empty)
        } else if self.library.is_none() || self.library.as_ref().unwrap().0.len() == 0 {
            Err(PkgIdError::MissingLibrary)
        } else if self.vendor.is_none() || self.vendor.as_ref().unwrap().0.len() == 0 {
            Err(PkgIdError::MissingVendor)
        } else {
            Ok(())
        }
    }

    pub fn get_name(&self) -> &PkgPart {
        &self.name
    }

    pub fn get_library(&self) -> &Option<PkgPart> {
        &self.library
    }

    pub fn get_vendor(&self) -> &Option<PkgPart> {
        &self.vendor
    }

    /// Transforms the `PkgId` is a `Vec` of its components.
    pub fn into_vec(self) -> Vec<Option<PkgPart>> {
        vec![Some(self.name), self.library, self.vendor]
    }

    /// Transforms into a complete `Vec`.
    /// 
    /// Errors if the PkgId is not fully qualified.
    pub fn into_full_vec(self) -> Result<Vec<PkgPart>, PkgIdError> {
        self.fully_qualified()?;
        Ok(vec![self.name, self.library.unwrap(), self.vendor.unwrap()])
    }
}

impl Display for PkgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{0}.{1}.{2}", 
            self.vendor.as_ref().unwrap_or(&PkgPart::new()), 
            self.library.as_ref().unwrap_or(&PkgPart::new()), 
            self.name)
    }
}

impl FromStr for PkgId {
    type Err = PkgIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        let chunks: Vec<&str> = s.rsplit('.').collect();
        if chunks.len() > 3 {
            return Err(PkgIdError::BadLen(s.to_owned(), chunks.len()));
        }

        Ok(PkgId {
            name: if let Some(&n) = chunks.get(0) {
                PkgPart::from_str(n)? } else { return Err(PkgIdError::Empty) },
            library: if let Some(&l) = chunks.get(1) {
                Some(PkgPart::from_str(l)?) } else { None },
            vendor: if let Some(&v) = chunks.get(2) {
                Some(PkgPart::from_str(v)?) } else { None }
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum PkgIdError {
    NotAlphabeticFirst(String),
    BadLen(String, usize),
    Empty,
    InvalidChar(String, char),
    MissingVendor,
    MissingLibrary,
}

impl Error for PkgIdError {}

impl Display for PkgIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use PkgIdError::*;
        match self {
            NotAlphabeticFirst(part) => write!(f, "pkgid part '{}' must begin with alphabetic character", part),
            BadLen(id, len) => write!(f, "bad length for pkgid '{}'; expecting 3 parts but found {}", id, len),
            InvalidChar(part, ch) => write!(f, "invalid character {} in pkgid part '{}'; can only contain alphanumeric characters, dashes, or underscores", ch, part),
            Empty => write!(f, "empty pkgid"),
            MissingLibrary => write!(f, "missing library part"),
            MissingVendor => write!(f, "missing vendor part"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn into_vec() {
        let p1 = PkgId::new()
            .name("NAME").unwrap()
            .library("library").unwrap()
            .vendor("Vendor").unwrap();

        assert_eq!(p1.into_vec(), vec![
            Some(PkgPart("NAME".to_owned())),
            Some(PkgPart("library".to_owned())),
            Some(PkgPart("Vendor".to_owned())),
        ]);

        let p1 = PkgId::new()
            .name("NAME").unwrap();

        assert_eq!(p1.into_vec(), vec![
            Some(PkgPart("NAME".to_owned())),
            None,
            None,
        ]);
    }

    #[test]
    fn new() {
        let pkgid = PkgId::new();
        assert_eq!(pkgid, PkgId {
            name: PkgPart::new(),
            library: None,
            vendor: None,
        });

        let pkgid = PkgId::new()
            .name("name").unwrap()
            .library("rary").unwrap()
            .vendor("vendor").unwrap();
        assert_eq!(pkgid, PkgId {
            name: PkgPart("name".to_owned()),
            library: Some(PkgPart("rary".to_owned())),
            vendor: Some(PkgPart("vendor".to_owned())),
        });

        assert_eq!(pkgid.get_name(), &PkgPart("name".to_owned()));
        assert_eq!(pkgid.get_library().as_ref().unwrap(), &PkgPart("rary".to_owned()));
        assert_eq!(pkgid.get_vendor().as_ref().unwrap(), &PkgPart("vendor".to_owned()));
    }

    #[test]
    fn equivalence() {
        let p1 = PkgId::new()
            .name("NAME").unwrap()
            .library("library").unwrap()
            .vendor("Vendor").unwrap();

        let p2 = PkgId::new()
            .name("name").unwrap()
            .library("LIBRARY").unwrap()
            .vendor("vendoR").unwrap();
        assert!(p1.equivalent(&p2));

        let p2 = PkgId::new()
            .name("name").unwrap()
            .library("library2").unwrap()
            .vendor("vendor").unwrap();
        assert_eq!(p1.equivalent(&p2), false);

        let p2 = PkgId::new()
            .name("name4").unwrap()
            .library("library").unwrap()
            .vendor("vendor").unwrap();
        assert_eq!(p1.equivalent(&p2), false);

        let p2 = PkgId::new()
            .name("name").unwrap()
            .library("library").unwrap()
            .vendor("ven_dor").unwrap();
        assert_eq!(p1.equivalent(&p2), false);

        // Converting '-' to '_' is applied for equivalence
        let p1 = PkgId::new()
            .name("name").unwrap()
            .library("lib_rary").unwrap()
            .vendor("Vendor").unwrap();

        let p2 = PkgId::new()
            .name("name").unwrap()
            .library("lib-rary").unwrap()
            .vendor("vendor").unwrap();
        assert_eq!(p1.equivalent(&p2), true);
    }

    #[test]
    fn validate() {
        //okays
        let s = "name";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "NAME_1";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "NAME_1-0";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));
        let s = "N9A-ME_1N--A432ME";
        assert_eq!(PkgPart::from_str(s), Ok(PkgPart(s.to_owned())));

        //errors
        assert!(PkgPart::from_str("ven dor").is_err());
        assert!(PkgPart::from_str("2name").is_err());
        assert!(PkgPart::from_str("_name").is_err());
        assert!(PkgPart::from_str("-name").is_err());
        assert!(PkgPart::from_str("path/name").is_err());
        assert!(PkgPart::from_str("na!me").is_err());
    }

    #[test]
    fn fully_qualified() {
        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().is_ok(), true);

        let pkgid = PkgId {
            vendor: Some(PkgPart("".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::MissingVendor);

        let pkgid = PkgId {
            vendor: None,
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::MissingVendor);

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("".to_owned())),
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::MissingLibrary);

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: None,
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::MissingLibrary);

        let pkgid = PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::Empty);

        let pkgid = PkgId {
            vendor: None,
            library: None,
            name: PkgPart("name".to_owned()),
        };
        assert_eq!(pkgid.fully_qualified().unwrap_err(), PkgIdError::MissingLibrary);
    }

    #[test]
    fn from_str() {
        assert_eq!(PkgId::from_str("vendor.library.name"), Ok(PkgId {
            vendor: Some(PkgPart("vendor".to_owned())),
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        }));

        assert_eq!(PkgId::from_str("library.name"), Ok(PkgId {
            vendor: None,
            library: Some(PkgPart("library".to_owned())),
            name: PkgPart("name".to_owned()),
        }));

        assert_eq!(PkgId::from_str("name"), Ok(PkgId {
            vendor: None,
            library: None,
            name: PkgPart("name".to_owned()),
        }));

        assert_eq!(PkgId::from_str(".name"), Ok(PkgId {
            vendor: None,
            library: Some(PkgPart("".to_owned())),
            name: PkgPart("name".to_owned()),
        }));

        assert_eq!(PkgId::from_str("..name"), Ok(PkgId {
            vendor: Some(PkgPart("".to_owned())),
            library: Some(PkgPart("".to_owned())),
            name: PkgPart("name".to_owned()),
        }));

        assert_eq!(PkgId::from_str("Ven-dor.Lib_Rary.name10"), Ok(PkgId {
            vendor: Some(PkgPart("Ven-dor".to_owned())),
            library: Some(PkgPart("Lib_Rary".to_owned())),
            name: PkgPart("name10".to_owned()),
        }));

        // invalid pkgids
        assert!(PkgId::from_str("vendor.library.name.").is_err());
        assert!(PkgId::from_str(".vendor.library.name").is_err());
        assert!(PkgId::from_str("vendor?.library.name").is_err());
        assert!(PkgId::from_str("vendor.library.name.extra").is_err());
        assert!(PkgId::from_str("0vendor.library.name").is_err());
        assert!(PkgId::from_str("vendor.0library.name").is_err());
        assert!(PkgId::from_str("vendor.library.0name").is_err());
        assert!(PkgId::from_str("vendor.library.name=").is_err());
        assert!(PkgId::from_str("vendor.lib rary.name").is_err());

        assert!(PkgId::from_str("v$ndor.library.name").is_err());
        assert!(PkgId::from_str("vendor.l*brary.name").is_err());
        assert!(PkgId::from_str("vendor.lib/rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib'rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib!rary.name").is_err());
        assert!(PkgId::from_str("vendor/library/name").is_err());
    }
    
}