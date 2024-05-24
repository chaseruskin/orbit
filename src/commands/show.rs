use crate::commands::helps::show;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::{Ip, PartialIpSpec};
use crate::core::lang::LangUnit;
use crate::core::version;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use std::cmp::Ordering;
use std::env::current_dir;

use cliproc::{cli, proc};
use cliproc::{Cli, Flag, Help, Positional, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Show {
    tags: bool,
    units: bool,
    ip: Option<PartialIpSpec>,
}

impl Subcommand<Context> for Show {
    fn construct<'c>(cli: &'c mut Cli) -> cli::Result<Self> {
        cli.check_help(Help::default().text(show::HELP))?;
        Ok(Show {
            tags: cli.check_flag(Flag::new("versions"))?,
            units: cli.check_flag(Flag::new("units"))?,
            ip: cli.check_positional(Positional::new("ip"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // collect all manifests available (load catalog)
        let catalog = Catalog::new()
            .installations(c.get_cache_path())?
            .downloads(c.get_downloads_path())?;

        let dev_ip: Option<Result<Ip, Fault>> = {
            match Context::find_ip_path(&current_dir().unwrap()) {
                Some(dir) => Some(Ip::load(dir, true)),
                None => None,
            }
        };
        let mut is_working_ip = false;

        // try to auto-determine the ip (check if in a working ip)
        let ip: &Ip = if let Some(spec) = &self.ip {
            // find the path to the provided ip by searching through the catalog
            if let Some(lvl) = catalog.inner().get(spec.get_name()) {
                // return the highest available version
                if let Some(slot) = lvl.get_install(spec.get_version()) {
                    slot
                } else {
                    // try to find from downloads
                    if let Some(slot) = lvl.get_download(spec.get_version()) {
                        slot
                    } else {
                        return Err(AnyError(format!("ip {} does not exist in the cache", spec)))?;
                    }
                }
            } else {
                return Err(AnyError(format!("no ip found anywhere")))?;
            }
        } else {
            if dev_ip.is_none() == true {
                return Err(AnyError(format!("no ip provided or detected")))?;
            } else {
                match &dev_ip {
                    Some(Ok(r)) => {
                        is_working_ip = true;
                        r
                    }
                    Some(Err(e)) => return Err(AnyError(format!("{}", e.to_string())))?,
                    _ => panic!("unreachable code"),
                }
            }
        };
        let _is_working_ip = is_working_ip;

        // load the ip's manifest
        if self.units == true {
            if ip.get_mapping().is_physical() == true {
                // force computing the primary design units if a physical ip (non-archived)
                let units = Ip::collect_units(
                    true,
                    &ip.get_root(),
                    &c.get_lang_mode(),
                    false,
                    ip.into_public_list(),
                )?;
                println!(
                    "{}",
                    Self::format_units_table(units.into_iter().map(|(_, unit)| unit).collect())
                );
            } else {
                // a 'virtual' ip, so try to extract units from
                println!(
                    "info: {}",
                    "unable to display HDL units from a downloaded IP; try again after installing"
                );
            }

            return Ok(());
        }

        // display all installed versions in the cache
        if self.tags == true {
            let specified_ver = self.ip.as_ref().unwrap().get_version().as_specific();

            return match catalog.get_possible_versions(ip.get_man().get_ip().get_name()) {
                Some(vers) => {
                    match vers.len() {
                        0 => {
                            println!("info: no versions in the cache")
                        }
                        _ => {
                            let mut data = String::new();
                            let header = format!(
                                "{:<10}{:<11}\n{2:->10}{2:->11}\n",
                                "Version", "Status", " ",
                            );
                            data.push_str(&header);
                            // further restrict versions if a particular version is set
                            vers.iter()
                                .filter(move |p| {
                                    specified_ver.is_none()
                                        || version::is_compatible(
                                            specified_ver.unwrap(),
                                            &p.get_version(),
                                        ) == true
                                })
                                .for_each(|v| {
                                    data.push_str(&format!(
                                        "{:<10}{:<11}\n",
                                        v.get_version().to_string(),
                                        v.get_state().to_string()
                                    ));
                                });
                            println!("{}", data);
                        }
                    }
                    Ok(())
                }
                None => Err(AnyError(format!("no ip found in catalog")))?,
            };
        }

        // print the manifest data "pretty"
        let s = toml::to_string_pretty(ip.get_man())?;
        println!("{}", s);
        Ok(())
    }
}

impl Show {
    /// Creates a string for to display the primary design units for the particular ip.
    fn format_units_table(table: Vec<LangUnit>) -> String {
        let header = format!(
            "\
{:<36}{:<14}{:<12}
{:->36}{3:->14}{3:->12}\n",
            "Identifier", "Type", "Visibility", " "
        );
        let mut body = String::new();

        let mut table = table;

        table.sort_by(|a, b| match a.get_visibility().cmp(&b.get_visibility()) {
            Ordering::Equal => a.get_name().cmp(&b.get_name()),
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        });

        for unit in table {
            body.push_str(&format!(
                "{:<36}{:<14}{:<12}\n",
                unit.get_name().to_string(),
                unit.to_string(),
                unit.get_visibility().to_string(),
            ));
        }

        header + &body
    }
}

// FUTURE FLAGS
// ============
// --changes                   view the changelog
// --readme                    view the readme
// --range <version:version>   narrow the displayed version list
