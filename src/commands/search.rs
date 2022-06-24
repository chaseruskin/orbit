use crate::Command;
use crate::FromCli;
use crate::core::ip::Ip;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct Search {
    ip: Option<PkgId>,
    cached: bool,
    developing: bool,
    available: bool,
}

impl Command for Search {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        let dev_path = c.get_development_path().unwrap();
        let cache_path = c.get_cache_path();
        let vendor_path = c.get_vendor_path();
        self.run((dev_path, cache_path, &vendor_path))
    }
}

use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::manifest::IpManifest;

/// Bundles the 3 levels: DEV_PATH, CACHE, and VENDORS
type Highway<'a> = (&'a PathBuf, &'a PathBuf, &'a PathBuf);

type IpNode = (Option<IpManifest>, Vec<IpManifest>, Vec<IpManifest>);

impl Search {
    /// Collects all `pkgid` in the user's universe: dev_path, cache, and availability through vendors.
    pub fn all_pkgid(paths: Highway) -> Result<HashMap<PkgId, IpNode>, Box<dyn std::error::Error>> {
        let mut set: HashMap<PkgId, IpNode> = HashMap::new();

        // collect development IP
        crate::core::manifest::IpManifest::detect_all(paths.0)?
            .into_iter()
            .for_each(|f| {
                set.insert(f.as_pkgid(), (Some(f), vec![], vec![]));
            });
        
        // collect cached IP
        crate::core::manifest::IpManifest::detect_all(paths.1)?
            .into_iter()
            .for_each(|f| {
                if let Some(entry) = set.get_mut(&f.as_pkgid()) {
                    entry.1.push(f);
                } else {
                    set.insert(f.as_pkgid(), (None, vec![f], vec![]));
                }
            });

        // collect available IP
        crate::core::vendor::VendorManifest::detect_all(paths.2)?
            .into_iter()
            .for_each(|f| {
                // read off the index table
                f.read_index()
                    .into_iter()
                    .for_each(|pkg| {
                    if let Some(entry) = set.get_mut(&pkg) {
                        // entry.1.push(Ip::from_manifest(pkg));
                        // @TODO find manifests in vendor dir
                    } else {
                         // @TODO find manifests in vendor dir
                        // set.insert(pkg.as_pkgid(), (None, vec![Ip::from_manifest(f)], vec![]));
                    }})
                });
        
        Ok(set)
    }

    fn run(&self, paths: Highway) -> Result<(), Box<dyn std::error::Error>> {
        let mut pkg_map: BTreeMap<PkgId, (bool, bool, bool)> = BTreeMap::new();

        let default = !(self.cached || self.developing || self.available);

        // collect development IP
        if default || self.developing {
            crate::core::manifest::IpManifest::detect_all(paths.0)?
            .into_iter()
            .for_each(|f| {
                pkg_map.insert(f.as_pkgid(), (true, false, false));
            });
        }
        
        // collect installed IP
        if default || self.cached {
            crate::core::manifest::IpManifest::detect_all(paths.1)?
            .into_iter()
            .for_each(|f| {
                let pkg = f.as_pkgid();
                if let Some(pair) = pkg_map.get_mut(&pkg) {
                    *pair = (pair.0, true, pair.2);
                } else {
                    pkg_map.insert(pkg, (false, true, false));
                }
            });
        }

        // collect available IP
        if default || self.available {
            crate::core::vendor::VendorManifest::detect_all(paths.2)?
                .into_iter()
                .for_each(|f| {
                    // read off the index table
                    f.read_index()
                        .into_iter()
                        .for_each(|pkg| {
                            if let Some(pair) = pkg_map.get_mut(&pkg) {
                                *pair = (pair.0, pair.1, true);
                            } else {
                                pkg_map.insert(pkg, (false, false, true));
                            }
                        });
                });
        }
        
        println!("{}", Self::fmt_table(pkg_map));
        Ok(())
    }

    fn fmt_table(catalog: BTreeMap<PkgId, (bool, bool, bool)>) -> String {
        let header = format!("\
{:<15}{:<15}{:<20}{:<9}
{:->15}{4:->15}{4:->20}{4:->9}\n", 
            "Vendor", "Library", "Name", "Status", " ");
        let mut body = String::new();
        for (ip, status) in catalog {
            body.push_str(&format!("{:<15}{:<15}{:<20}{:<2}{:<2}{:<2}\n", 
                ip.get_vendor().as_ref().unwrap().to_string(),
                ip.get_library().as_ref().unwrap().to_string(),
                ip.get_name().to_string(),
                { if status.0 { "D" } else { "" } },
                { if status.1 { "I" } else { "" } },
                { if status.2 { "A" } else { "" } },
            ));
        }
        header + &body
    }
}

impl FromCli for Search {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Search {
            ip: cli.check_positional(Positional::new("ip"))?,
            cached: cli.check_flag(Flag::new("install").switch('i'))?,
            developing: cli.check_flag(Flag::new("develop").switch('d'))?,
            available: cli.check_flag(Flag::new("available").switch('a'))?,
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
    --develop, -d       filter for ip in-development
    --available, -a     filter for ip available from vendors

Use 'orbit help search' to learn more about the command.
";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fmt_table() {
        let t = Search::fmt_table(BTreeMap::new());
        let table = "\
Vendor         Library        Name                Status   
-------------- -------------- ------------------- -------- 
";
        assert_eq!(t, table);
    }
}