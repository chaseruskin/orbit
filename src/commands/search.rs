use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Flag};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::core::pkgid::PkgId;

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
        self.run(dev_path)
    }
}

impl Search {
    fn run(&self, dev_path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let ips = crate::core::manifest::find_dev_manifests(dev_path)?;
        println!("{}", Self::fmt_table(ips));
        Ok(())
        // find all ip installed in cache

        // walk vendor directory to find all ip manifest available
    }

    fn fmt_table(catalog: Vec<crate::core::manifest::Manifest>) -> String {
        let header = format!("\
{:<15}{:<15}{:<20}{:<9}
{:->15}{4:->15}{4:->20}{4:->9}\n", 
            "Vendor", "Library", "Name", "Status", " ");
        let mut body = String::new();
        for ip in catalog {
            body.push_str(&format!("{:<15}{:<15}{:<20}\n", 
                ip.get_doc()["ip"]["vendor"].as_str().unwrap(),
                ip.get_doc()["ip"]["library"].as_str().unwrap(),
                ip.get_doc()["ip"]["name"].as_str().unwrap(),
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
            cached: cli.check_flag(Flag::new("cache").switch('c'))?,
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
    --cache, -c         filter for ip installed to cache
    --develop, -d       filter for ip in-development
    --available, -a     filter for ip available from vendors

Use 'orbit help search' to learn more about the command.
";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fmt_table() {
        let t = Search::fmt_table(vec![]);
        let table = "\
Vendor         Library        Name                Status   
-------------- -------------- ------------------- -------- 
";
        assert_eq!(t, table);
    }
}