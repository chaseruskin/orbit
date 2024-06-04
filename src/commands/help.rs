use crate::commands::manuals;
use crate::util::anyerror::AnyError;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Help {
    topic: Option<Topic>,
}

impl Subcommand<()> for Help {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(cliproc::Help::with(HELP))?;
        Ok(Help {
            topic: cli.get(Arg::positional("topic"))?,
        })
    }

    fn execute(self, _: &()) -> proc::Result {
        self.run()?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum Topic {
    New,
    Plan,
    Build,
    Launch,
    Download,
    Install,
    Tree,
    Search,
    Get,
    Init,
    Show,
    Env,
    Config,
    Uninstall,
    Read,
}

impl std::str::FromStr for Topic {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "env" => Self::Env,
            "new" => Self::New,
            "plan" => Self::Plan,
            "build" => Self::Build,
            "search" => Self::Search,
            "launch" => Self::Launch,
            "download" => Self::Download,
            "install" => Self::Install,
            "tree" => Self::Tree,
            "get" => Self::Get,
            "init" => Self::Init,
            "show" => Self::Show,
            "config" => Self::Config,
            "uninstall" => Self::Uninstall,
            "read" => Self::Read,
            _ => return Err(AnyError(format!("topic '{}' not found", s))),
        })
    }
}

impl Topic {
    /// Transforms the variant to its corresponding manual page.
    fn as_manual(&self) -> &str {
        use Topic::*;
        match &self {
            Env => manuals::env::MANUAL,
            Show => manuals::show::MANUAL,
            Get => manuals::get::MANUAL,
            Tree => manuals::tree::MANUAL,
            Download => manuals::download::MANUAL,
            New => manuals::new::MANUAL,
            Plan => manuals::plan::MANUAL,
            Search => manuals::search::MANUAL,
            Build => manuals::build::MANUAL,
            Launch => manuals::launch::MANUAL,
            Install => manuals::install::MANUAL,
            Init => manuals::init::MANUAL,
            Config => manuals::config::MANUAL,
            Uninstall => manuals::remove::MANUAL,
            Read => manuals::read::MANUAL,
        }
    }
}

impl Help {
    fn run(&self) -> Result<(), AnyError> {
        let contents = match &self.topic {
            Some(t) => t.as_manual(),
            None => manuals::orbit::MANUAL,
        };
        // @todo/idea: check for a pager program to pipe contents into?
        println!("{}", contents);
        Ok(())
    }
}

const HELP: &str = "\
Read in-depth documentation around Orbit topics.

Usage:
    orbit help [<topic>]

Args:
    <topic>         a listed topic or any orbit subcommand

Topics:
    toml            learn about .toml files
    cache           learn about orbit's caching system
    manifest        learn about the Orbit.toml file
    template        learn about templates
    blueprint       learn about generating a pre-build data file
    vendor          learn about hosting multiple ip together

Use 'orbit help --list' to see all available topics.
";
