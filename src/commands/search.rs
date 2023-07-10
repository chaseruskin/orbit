use crate::core::context::Context;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::Fault;
use crate::OrbitResult;
use clif::arg::{Flag, Optional, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use std::collections::BTreeMap;

use crate::core::catalog::Catalog;
use crate::core::catalog::IpLevel;
use crate::core::version::AnyVersion;

#[derive(Debug, PartialEq)]
pub struct Search {
    ip: Option<PkgPart>,
    cached: bool,
    downloaded: bool,
    keywords: Vec<String>,
    limit: Option<usize>,
    hard_match: bool,
}

impl Command<Context> for Search {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        let default = !(self.cached || self.downloaded);
        let mut catalog = Catalog::new();

        // collect installed IP
        if default || self.cached {
            catalog = catalog.installations(c.get_cache_path())?;
        }

        // collect downloaded IP
        if default || self.downloaded {
            catalog = catalog.downloads(c.get_downloads_path())?;
        }

        self.run(&catalog)
    }
}

impl Search {
    fn run(&self, catalog: &Catalog) -> Result<(), Fault> {
        // transform into a BTreeMap for alphabetical ordering
        let mut tree = BTreeMap::new();
        catalog
            .inner()
            .into_iter()
            // filter by name if user entered a pkgid to search
            .filter(|(key, iplvl)| { 
                if let Some(prj) = iplvl.get(true, &AnyVersion::Latest) {
                    match self.hard_match {
                        true => {
                            let name_match = match &self.ip {
                                // names must be identical
                                Some(pkgid) => {
                                    if key == &pkgid {
                                        true
                                    } else {
                                        false
                                    }
                                }
                                // move on to the keywords
                                None => true,
                            };
                            let keyword_match = {
                                for kw in &self.keywords {
                                    if prj.get_man().get_ip().get_keywords().contains(kw) == false {
                                        return false;
                                    }
                                }
                                true
                            };
                            name_match && keyword_match
                        }
                        false => {
                            // pass everything if there is no filters applied
                            if self.ip.is_none() && self.keywords.is_empty() {
                                return true;
                            }
                            // try to match the name of the IP with ones in the database
                            let name_match = match &self.ip {
                                // names must be identical
                                Some(pkgid) => key.starts_with(&pkgid),
                                // move on to the keywords
                                None => false,
                            };
                            // try to evaluate keywords
                            if name_match == false {
                                for kw in &self.keywords {
                                    if prj.get_man().get_ip().get_keywords().contains(kw) == true {
                                        return true;
                                    }
                                }
                                false
                            } else {
                                true
                            }
                        }
                    }
                } else {
                    false
                }
            })
            .for_each(|(key, status)| {
                tree.insert(key, status);
            });

        println!("{}", Self::fmt_table(tree, self.limit));
        Ok(())
    }

    fn fmt_table(catalog: BTreeMap<&PkgPart, &IpLevel>, limit: Option<usize>) -> String {
        let header = format!(
            "\
{:<28}{:<10}{:<9}
{3:->28}{3:->10}{3:->11}\n",
            "Package", "Latest", "Status", " "
        );
        let mut body = String::new();
        let mut index = 0;
        for (ip, status) in catalog {
            if let Some(cap) = limit {
                index += 1;
                // exit when next entry will go past the max results
                if index > cap {
                    break;
                }
            }

            body.push_str(&format!(
                "{:<28}{:<10}     {:<9}\n",
                ip.to_string(),
                status
                    .get(true, &AnyVersion::Latest)
                    .unwrap()
                    .get_man()
                    .get_ip()
                    .get_version(),
                if status.is_installed() == true {
                    "Installed"
                } else if status.is_downloaded() == true {
                    "Downloaded"
                } else {
                    ""
                },
            ));
        }
        header + &body
    }
}

impl FromCli for Search {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Search {
            downloaded: cli.check_flag(Flag::new("download").switch('d'))?,
            cached: cli.check_flag(Flag::new("install").switch('i'))?,
            hard_match: cli.check_flag(Flag::new("match"))?,
            limit: cli.check_option(Optional::new("limit"))?,
            keywords: cli
                .check_option_all(Optional::new("keyword").value("term"))?
                .unwrap_or(Vec::new()),
            ip: cli.check_positional(Positional::new("ip"))?,
        });
        command
    }
}

const HELP: &str = "\
Browse and find ip from the catalog.

Usage:
    orbit search [options] [<ip>]

Args:
    <ip>                a partially qualified pkgid to lookup ip

Options:
    --install, -i       filter for ip installed to cache
    --download, -d      filter for ip downloaded to the queue
    --keyword <term>... special word to filter out packages
    --limit <count>     maximum number of results to return
    --match             return results with each filter passed

Use 'orbit help search' to learn more about the command.
";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fmt_table() {
        let t = Search::fmt_table(BTreeMap::new(), None);
        let table = "\
Package                     Latest    Status   
--------------------------- --------- ---------- 
";
        assert_eq!(t, table);
    }
}
