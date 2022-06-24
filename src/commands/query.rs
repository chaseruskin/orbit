use std::collections::BTreeMap;

use crate::Command;
use crate::FromCli;
use crate::core::pkgid::PkgId;
use crate::core::version::Version;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag, Optional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Query {
    ip: PkgId,
    tags: bool,
}

impl FromCli for Query {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Query {
            tags: cli.check_flag(Flag::new("tags"))?,
            ip: cli.require_positional(Positional::new("ip"))?,
        });
        command
    }
}

impl Command for Query {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, _c: &Context) -> Result<(), Self::Err> {
        // collect all ip in the user's universe to see if ip exists
        if self.tags == true {
            println!("{}", format_version_table((None, vec![], vec![])));
            return Ok(())
        }
        self.run()
    }
}

impl Query {
    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Tracks the dev version, installed versions, and available versions
type VersionTable = (Option<Version>, Vec<Version>, Vec<Version>);

/// Creates a string for a version table for the particular ip.
fn format_version_table(table: VersionTable) -> String {
    let header = format!("\
{:<15}{:<9}
{:->15}{2:->9}\n",
                "Version", "Status", " ");
    // create a hashset of all available versions to form a list
    let mut btmap = BTreeMap::<Version, (bool, bool, bool)>::new();
    // log what version the dev ip is at
    if let Some(v) = table.0 {
        btmap.insert(v, (true, false, false));
    }
    // log the installation versions
    for v in table.1 {
        match btmap.get_mut(&v) {
            Some(entry) => entry.1 = true,
            None => { btmap.insert(v, (false, true, false)); () },
        }
    }
    // log the available versions
    for v in table.2 {
        match btmap.get_mut(&v) {
            Some(entry) => entry.1 = true,
            None => { btmap.insert(v, (false, false, true)); () },
        } 
    }
    // create body text
    let mut body = String::new();
    for (ver, status) in btmap {
        body.push_str(&format!("{:<15}{:<2}{:<2}{:<2}\n", 
            ver.to_string(),
            { if status.0 { "D" } else { "" } },
            { if status.1 { "I" } else { "" } },
            { if status.2 { "A" } else { "" } },
        ));
    }
    header + &body
}

const HELP: &str = "\
Probe information about an ip

Usage:
    orbit query [options] <ip>

Args:
    <ip>               the pkgid to request data about

Options:
    --tags
    --range <version:version>
    --install, -i <version>
    --available, -a <version>
    --develop, -d,
    --units

Use 'orbit help query' to learn more about the command.
";