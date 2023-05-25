use std::str::FromStr;
use crate::util::anyerror::AnyError;
use crate::core::v2::protocol::Name;

type ProtocolName = Name;

/// A [Source] outlines the process and location for extracting packages from the internet.
#[derive(Debug, PartialEq)]
pub struct Source {
    protocol: Option<ProtocolName>,
    url: String,
}

pub const DELIM: &str = "+";

impl FromStr for Source {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // check if the delimiter exists
        match s.split_once(DELIM) {
            Some((proto, url)) => { 
                Ok(Self {
                    protocol: Some(ProtocolName::from_str(proto)?),
                    url: String::from(url),
                })
            },
            None => {
                Ok(Self {
                    protocol: None,
                    url: String::from(s),
                })
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let src: &str = "cfs+https://some.url";

        assert_eq!(Source::from_str(src).unwrap(), Source {
            protocol: Some(ProtocolName::from_str("cfs").unwrap()),
            url: String::from("https://some.url"),
        });

        let src: &str = "https://some.url";

        assert_eq!(Source::from_str(src).unwrap(), Source {
            protocol: None,
            url: String::from("https://some.url"),
        });
    }
}