//! File     : pkgid.rs
//! Abstract :
//!     A `pkgid` is formed from a string following VLNV format.

use std::str::FromStr;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
struct PkgId {
    vendor: Option<String>,
    library: Option<String>,
    name: String
}

impl Display for PkgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{0}.{1}.{2}", 
            self.vendor.as_ref().unwrap_or(&"".to_owned()), 
            self.library.as_ref().unwrap_or(&"".to_owned()), 
            self.name)
    }
}

impl PkgId {
    pub fn new() -> Self {
        PkgId {
            vendor: None,
            library: None,
            name: "".to_owned(),
        }
    }

    pub fn name(mut self, n: &str) -> Result<Self, PkgIdError> {
        self.name = PkgId::validate_part(n)?.to_owned();
        Ok(self)
    }

    pub fn library(mut self, l: &str) -> Result<Self, PkgIdError> {
        self.library = Some(PkgId::validate_part(l)?.to_owned());
        Ok(self)
    }

    pub fn vendor(mut self, v: &str) -> Result<Self, PkgIdError> {
        self.vendor = Some(PkgId::validate_part(v)?.to_owned());
        Ok(self)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_library(&self) -> &Option<String> {
        &self.library
    }

    pub fn get_vendor(&self) -> &Option<String> {
        &self.vendor
    }

    /// Verify a part follows the spec
    fn validate_part<'a>(s: &'a str) -> Result<&'a str, PkgIdError> {
        use PkgIdError::*;

        if let Some(c) = s.chars().next() {
            if c.is_alphabetic() == false {
                return Err(NotAlphabeticFirst(s.to_owned()));
            }
        }
        // find first char in pkgid part not following spec
        let result = s.chars()
            .find(|&c| {
                !c.is_alphanumeric() && !(c == '_') && !(c == '-')
            }
        );
        if let Some(r) = result {
            Err(InvalidChar(s.to_owned(), r))
        } else {
            Ok(s)
        }
    }

    /// Checks if all the parts for a `PkgId` are specified and nonempty.
    pub fn fully_qualified(&self) -> bool {
        self.name.len() > 0 && 
        self.vendor.is_some() && self.vendor.as_ref().unwrap().len() > 0 &&
        self.library.is_some() && self.library.as_ref().unwrap().len() > 0
    }
}

#[derive(Debug, PartialEq)]
enum PkgIdError {
    NotAlphabeticFirst(String),
    BadLen(String, usize),
    Empty,
    InvalidChar(String, char),
}

impl Error for PkgIdError {}

impl Display for PkgIdError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use PkgIdError::*;
        match self {
            NotAlphabeticFirst(part) => write!(f, "pkgid part\"{}\" must begin with alphabetic character", part),
            BadLen(id, len) => write!(f, "bad length for pkgid \"{}\"; expecting 3 parts but found {}", id, len),
            InvalidChar(part, ch) => write!(f, "invalid character {} in pkgid part \"{}\"; can only contain alphanumeric characters, dashes, or underscores", ch, part),
            Empty => write!(f, "empty pkgid"),
        }
    }
}

impl FromStr for PkgId {
    type Err = PkgIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        use PkgIdError::*;
        
        let chunks: Vec<&str> = s.rsplit_terminator('.').collect();

        if chunks.len() > 3 {
            return Err(BadLen(s.to_owned(), chunks.len()));
        }

        Ok(PkgId {
            name: if let Some(&n) = chunks.get(0) {
                PkgId::validate_part(n)?.to_owned() } else { return Err(Empty) },
            library: if let Some(&l) = chunks.get(1) {
                Some(PkgId::validate_part(l)?.to_owned()) } else { None },
            vendor: if let Some(&v) = chunks.get(2) {
                Some(PkgId::validate_part(v)?.to_owned()) } else { None }
            }
        )
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let pkgid = PkgId::new();
        assert_eq!(pkgid, PkgId {
            name: "".to_owned(),
            library: None,
            vendor: None,
        });

        let pkgid = PkgId::new()
            .name("name").unwrap()
            .library("rary").unwrap()
            .vendor("vendor").unwrap();
        assert_eq!(pkgid, PkgId {
            name: "name".to_owned(),
            library: Some("rary".to_owned()),
            vendor: Some("vendor".to_owned()),
        });
    }


    #[test]
    fn validate() {
        //okays
        let s = "name";
        assert_eq!(PkgId::validate_part(s), Ok(s));
        let s = "NAME_1";
        assert_eq!(PkgId::validate_part(s), Ok(s));
        let s = "NAME_1-0";
        assert_eq!(PkgId::validate_part(s), Ok(s));
        let s = "N9A-ME_1N--A432ME";
        assert_eq!(PkgId::validate_part(s), Ok(s));

        //errors
        assert!(PkgId::validate_part("2name").is_err());
        assert!(PkgId::validate_part("_name").is_err());
        assert!(PkgId::validate_part("-name").is_err());
        assert!(PkgId::validate_part("path/name").is_err());
        assert!(PkgId::validate_part("na!me").is_err());
    }

    #[test]
    fn fully_qualified() {
        let pkgid = PkgId {
            vendor: Some("vendor".to_owned()),
            library: Some("library".to_owned()),
            name: "name".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), true);

        let pkgid = PkgId {
            vendor: Some("".to_owned()),
            library: Some("library".to_owned()),
            name: "name".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), false);

        let pkgid = PkgId {
            vendor: None,
            library: Some("library".to_owned()),
            name: "name".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), false);

        let pkgid = PkgId {
            vendor: Some("vendor".to_owned()),
            library: Some("".to_owned()),
            name: "name".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), false);

        let pkgid = PkgId {
            vendor: Some("vendor".to_owned()),
            library: None,
            name: "name".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), false);

        let pkgid = PkgId {
            vendor: Some("vendor".to_owned()),
            library: Some("library".to_owned()),
            name: "".to_owned(),
        };
        assert_eq!(pkgid.fully_qualified(), false);
    }

    #[test]
    fn from_str() {
        assert_eq!(PkgId::from_str("vendor.library.name"), Ok(PkgId {
            vendor: Some("vendor".to_owned()),
            library: Some("library".to_owned()),
            name: "name".to_owned(),
        }));

        assert_eq!(PkgId::from_str("library.name"), Ok(PkgId {
            vendor: None,
            library: Some("library".to_owned()),
            name: "name".to_owned(),
        }));

        assert_eq!(PkgId::from_str("name"), Ok(PkgId {
            vendor: None,
            library: None,
            name: "name".to_owned(),
        }));

        assert_eq!(PkgId::from_str(".name"), Ok(PkgId {
            vendor: None,
            library: Some("".to_owned()),
            name: "name".to_owned(),
        }));

        assert_eq!(PkgId::from_str("..name"), Ok(PkgId {
            vendor: Some("".to_owned()),
            library: Some("".to_owned()),
            name: "name".to_owned(),
        }));

        assert_eq!(PkgId::from_str("Ven-dor.Lib_Rary.name10"), Ok(PkgId {
            vendor: Some("Ven-dor".to_owned()),
            library: Some("Lib_Rary".to_owned()),
            name: "name10".to_owned(),
        }));

        // invalid pkgids
        assert!(PkgId::from_str("vendor?.library.name").is_err());
        assert!(PkgId::from_str("vendor.library.name.extra").is_err());
        assert!(PkgId::from_str("0vendor.library.name").is_err());
        assert!(PkgId::from_str("vendor.0library.name").is_err());
        assert!(PkgId::from_str("vendor.library.0name").is_err());
        assert!(PkgId::from_str("vendor.library.name=").is_err());

        assert!(PkgId::from_str("v$ndor.library.name").is_err());
        assert!(PkgId::from_str("vendor.l*brary.name").is_err());
        assert!(PkgId::from_str("vendor.lib/rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib'rary.name").is_err());
        assert!(PkgId::from_str("vendor.lib!rary.name").is_err());
        assert!(PkgId::from_str("vendor/library/name").is_err());
    }
    
}