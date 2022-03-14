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

#[derive(Debug, PartialEq)]
enum PkgIdError {
    NotAlphabeticFirst(String, String),
    BadLen(String, usize),
    Empty,
    InvalidChar(String, char),
}

impl Error for PkgIdError {}

impl Display for PkgIdError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use PkgIdError::*;
        match self {
            NotAlphabeticFirst(id, part) => write!(f, "pkgid {} is missing leading alphabetic character for part {}", id, part),
            BadLen(id, len) => write!(f, "bad length for pkgid {}; expecting 3 parts but found {}", id, len),
            InvalidChar(id, ch) => write!(f, "invalid character {} in pkgid {}; can only contain alphanumeric characters, dashes, or underscores", ch, id),
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

        let validate = |p: &str| {
            if let Some(c) = p.chars().next() {
                if c.is_alphabetic() == false {
                    return Err(NotAlphabeticFirst(s.to_owned(), p.to_owned()));
                }
            }
            // find first char in pkgid part not following spec
            let result = p.chars()
                .find(|&c| {
                    !c.is_alphanumeric() && !(c == '_') && !(c == '-')
                }
            );
            if let Some(r) = result {
                return Err(InvalidChar(p.to_owned(), r));
            }

            Ok(p.to_owned())
        };

        Ok(PkgId {
            name: if let Some(&n) = chunks.get(0) {
                validate(n)? } else { return Err(Empty) },
            library: if let Some(&l) = chunks.get(1) {
                Some(validate(l)?) } else { None },
            vendor: if let Some(&v) = chunks.get(2) {
                Some(validate(v)?) } else { None }
            }
        )
    }
}


#[cfg(test)]
mod test {
    use super::*;

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