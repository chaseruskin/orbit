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

use crate::core::context::Context;
use crate::core::ip::Mapping;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::Fault;
use std::collections::BTreeMap;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

use crate::commands::helps::search;
use crate::core::catalog::Catalog;
use crate::core::catalog::IpLevel;
use crate::core::version::AnyVersion;

#[derive(Debug, PartialEq)]
pub struct Search {
    ip: Option<PkgPart>,
    cached: bool,
    downloaded: bool,
    available: bool,
    keywords: Vec<String>,
    limit: Option<usize>,
    hard_match: bool,
}

impl Subcommand<Context> for Search {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(search::HELP))?;
        Ok(Search {
            downloaded: cli.check(Arg::flag("download").switch('d'))?,
            cached: cli.check(Arg::flag("install").switch('i'))?,
            available: cli.check(Arg::flag("available").switch('a'))?,
            hard_match: cli.check(Arg::flag("match"))?,
            limit: cli.get(Arg::option("limit").value("num"))?,
            keywords: cli
                .get_all(Arg::option("keyword").value("term"))?
                .unwrap_or(Vec::new()),
            ip: cli.get(Arg::positional("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        let mut catalog = Catalog::new();
        // collect installed IP
        catalog = catalog.installations(c.get_cache_path())?;
        // collect downloaded IP
        catalog = catalog.downloads(c.get_downloads_path())?;
        // collect available IP
        catalog = catalog.available(&c.get_config().get_channels())?;

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
                if let Some(prj) = iplvl.get(true, true, &AnyVersion::Latest) {
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

        println!(
            "{}",
            Self::fmt_table(
                tree,
                self.limit,
                self.cached,
                self.downloaded,
                self.available
            )
        );
        Ok(())
    }

    fn fmt_table(
        catalog: BTreeMap<&PkgPart, &IpLevel>,
        limit: Option<usize>,
        cached: bool,
        downloaded: bool,
        available: bool,
    ) -> String {
        //         let header = format!(
        //             "\
        // {:<28}{:<10}{:<9}
        // {3:->28}{3:->10}{3:->11}\n",
        //             "Ip", "Latest", "Status", " "
        //         );
        let header = String::new();
        let mut body = String::new();
        let mut index = 0;

        let default = !(cached || downloaded || available);

        // note: There is definitely a nicer way to handle all of this logic... but this works for now.

        for (name, status) in catalog {
            // use this variable to determine if a level higher in the catalog has a higher version not displayed right now
            let mut is_update_available = false;
            // return the highest version (return installation when they are equal in downloads and cache)
            let ip = {
                let ins = status.get_install(&AnyVersion::Latest);
                let dld = status.get_download(&AnyVersion::Latest);
                let ava = status.get_available(&AnyVersion::Latest);
                // prioritize who display
                if let Some(installed_ip) = ins {
                    // check if download or available have a later version
                    if let Some(download_ip) = dld {
                        if default || cached {
                            is_update_available = download_ip.get_man().get_ip().get_version()
                                > installed_ip.get_man().get_ip().get_version();
                        }
                    }
                    // if update is not coming from download, see if it comes from available
                    if is_update_available == false {
                        if let Some(avail_ip) = ava {
                            if default || cached || downloaded && dld.is_some() {
                                is_update_available = avail_ip.get_man().get_ip().get_version()
                                    > installed_ip.get_man().get_ip().get_version();
                            }
                        }
                    }

                    // determine who to display
                    match default || cached {
                        true => Some(installed_ip),
                        false => match downloaded {
                            true => dld,
                            false => ava,
                        },
                    }
                } else if let Some(download_ip) = dld {
                    if let Some(avail_ip) = ava {
                        if default || downloaded {
                            is_update_available = avail_ip.get_man().get_ip().get_version()
                                > download_ip.get_man().get_ip().get_version();
                        }
                    }
                    match default || downloaded {
                        true => Some(download_ip),
                        false => ava,
                    }
                } else {
                    ava
                }
            };
            // ip should NOT be empty but skip if it is
            let ip = match ip {
                Some(r) => r,
                None => continue,
            };

            if let Some(cap) = limit {
                index += 1;
                // exit when next entry will go past the max results
                if index > cap {
                    break;
                }
            }

            // determine if to skip this ip based on settings
            let display_to_screen = match ip.get_mapping() {
                Mapping::Physical => default == true || cached == true,
                Mapping::Virtual(_) => default == true || downloaded == true,
                Mapping::Imaginary => default == true || available == true,
                Mapping::Relative => false,
            };
            if display_to_screen == false {
                continue;
            }

            body.push_str(&format!(
                "{:<28}{:<10}{:<9}\n",
                name.to_string(),
                ip.get_man().get_ip().get_version().to_string() + {
                    if is_update_available == true {
                        "*"
                    } else {
                        ""
                    }
                },
                match ip.get_mapping() {
                    Mapping::Physical => "install",
                    Mapping::Virtual(_) => "download",
                    Mapping::Imaginary => "available",
                    Mapping::Relative => "local",
                },
            ));
        }
        header + &body
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn fmt_table() {
//         let t = Search::fmt_table(BTreeMap::new(), None, false, false);
//         let table = "\
// Package                     Latest    Status
// --------------------------- --------- ----------
// ";
//         assert_eq!(t, table);
//     }
// }
