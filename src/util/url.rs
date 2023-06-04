use serde_derive::{Deserialize, Serialize};
use std::{num::ParseIntError, str::FromStr};
use url::ParseError;
use url::Url as CrateUrl;

use super::strcmp;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize, Serialize)]
pub enum Url {
    #[serde(rename = "https")]
    Https(Https),
    #[serde(rename = "ssh")]
    Ssh(Ssh),
}

impl Url {
    /// Casts the url to https.
    pub fn as_https(&self) -> Https {
        match self {
            Self::Https(url) => url.to_owned(),
            Self::Ssh(url) => url.to_https(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UrlError {
    SshError(SshError),
    HttpsError(ParseError),
}

impl std::error::Error for UrlError {}

impl std::fmt::Display for UrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SshError(e) => write!(f, "{}", e),
            Self::HttpsError(e) => write!(f, "{}", e),
        }
    }
}

impl FromStr for Url {
    type Err = UrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // check for https:// base
        if let Some((base, _)) = s.split_once("://") {
            if strcmp::cmp_ascii_ignore_case(base, "https")
                || strcmp::cmp_ascii_ignore_case(base, "http")
            {
                match Https::from_str(s) {
                    Ok(r) => Ok(Self::Https(r)),
                    Err(e) => Err(Self::Err::HttpsError(e)),
                }
            } else {
                match Ssh::from_str(s) {
                    Ok(r) => Ok(Self::Ssh(r)),
                    Err(e) => Err(Self::Err::SshError(e)),
                }
            }
        } else {
            match Ssh::from_str(s) {
                Ok(r) => Ok(Self::Ssh(r)),
                Err(e) => Err(Self::Err::SshError(e)),
            }
        }
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Https(url) => url.fmt(f),
            Self::Ssh(url) => url.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Https(String);

impl std::str::FromStr for Https {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(CrateUrl::from_str(s)?.to_string()))
    }
}

impl std::fmt::Display for Https {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, Deserialize, Serialize)]
/// SSH ::= [ssh://]<user>@<host>:[port]</path/to/repo> or {user}@<host>:<path/to/repo>
pub struct Ssh {
    prefix: Option<String>,
    user: String,
    host: String,
    port: Option<u16>,
    path: String,
}

impl Ssh {
    fn to_https(&self) -> Https {
        Https(
            CrateUrl::from_str(&format!("https://{}/{}", self.host, self.path))
                .unwrap()
                .to_string(),
        )
    }
}

impl std::str::FromStr for Ssh {
    type Err = SshError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // check if the string begins with ssh://
        let (prefix, url) = if let Some((base, url)) = s.split_once("://") {
            match strcmp::cmp_ascii_ignore_case(base, "ssh") {
                true => (Some(base.to_owned()), url),
                false => return Err(Self::Err::BadBase(base.to_owned())),
            }
        } else {
            (None, s)
        };
        // parse user component
        let (user, url) = match url.split_once('@') {
            Some((user, url)) => (user.to_owned(), url),
            None => return Err(Self::Err::MissingHost),
        };
        // parse host component
        let (host, url) = match url.split_once(':') {
            Some((host, url)) => (host.to_owned(), url),
            None => match prefix.is_some() {
                true => return Err(Self::Err::MissingPort),
                false => return Err(Self::Err::MissingPath),
            },
        };
        // parse port component
        let (port, url) = match prefix.is_some() {
            true => match url.split_once('/') {
                Some((port, path)) => (
                    Some(match port.parse() {
                        Ok(r) => r,
                        Err(e) => return Err(Self::Err::BadPort(port.to_owned(), e)),
                    }),
                    path,
                ),
                None => return Err(Self::Err::MissingPort),
            },
            false => (None, url),
        };

        Ok(Self {
            prefix: prefix,
            user: user,
            host: host,
            port: port,
            path: url.to_owned(),
        })
    }
}

impl std::fmt::Display for Ssh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(base) = &self.prefix {
            write!(f, "{}://", base)?;
        }
        write!(f, "{}@{}:", self.user, self.host)?;
        if let Some(port) = &self.port {
            write!(f, "{}/", port)?;
        }
        write!(f, "{}", self.path)
    }
}

#[derive(Debug, PartialEq)]
pub enum SshError {
    BadBase(String),
    MissingHost, // no '@' symbol
    MissingPort, // no ':'
    MissingPath,
    BadPort(String, ParseIntError), // port not a 16-bit number
}

impl std::error::Error for SshError {}

impl std::fmt::Display for SshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadBase(s) => write!(f, "invalid ssh base '{}'", s),
            Self::MissingHost => write!(f, "missing '@' symbol to define host"),
            Self::MissingPort => write!(f, "missing ':' symbol proceeding host"),
            Self::MissingPath => write!(f, "missing path"),
            Self::BadPort(num, err) => write!(f, "invalid port number '{}' due to {}", num, err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn ssh_from_str() {
        // positive case
        assert_eq!(
            Ssh::from_str("ssh://git@github.ks-tech.org:22/rary/gates.git").is_ok(),
            true
        );
        // missing port number
        assert_eq!(
            Ssh::from_str("ssh://git@github.ks-tech.org:/rary/gates.git").is_err(),
            true
        );
        // positive case without base
        assert_eq!(
            Ssh::from_str("git@github.ks-tech.org:rary/gates.git").is_ok(),
            true
        );
    }

    #[test]
    fn ssh_to_str() {
        // positive case without base
        assert_eq!(
            Ssh::from_str("git@github.ks-tech.org:rary/gates.git")
                .unwrap()
                .to_string(),
            String::from("git@github.ks-tech.org:rary/gates.git")
        );
        // positive case with base
        assert_eq!(
            Ssh::from_str("ssh://git@github.ks-tech.org:22/rary/gates.git")
                .unwrap()
                .to_string(),
            String::from("ssh://git@github.ks-tech.org:22/rary/gates.git")
        );
    }

    #[test]
    fn https_from_str() {
        let url = Https::from_str("https://github.ks-tech.org/rary/gates.git").unwrap();
        assert_eq!(
            CrateUrl::from_str(&url.0).unwrap().path(),
            "/rary/gates.git"
        );
        assert_eq!(
            CrateUrl::from_str(&url.0).unwrap().host_str().unwrap(),
            "github.ks-tech.org"
        );
    }

    #[test]
    fn ssh_to_https() {
        let url = Ssh::from_str("git@github.ks-tech.org:rary/gates.git").unwrap();
        assert_eq!(
            url.to_https(),
            Https::from_str("https://github.ks-tech.org/rary/gates.git").unwrap()
        )
    }
}
