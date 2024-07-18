use crate::commands::manuals;
use crate::util::anyerror::AnyError;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Subcommand};

#[derive(Debug, PartialEq)]
pub struct Help {
    list: bool,
    topic: Option<Topic>,
}

impl Subcommand<()> for Help {
    fn interpret<'c>(cli: &'c mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(cliproc::Help::with(HELP))?;
        Ok(Help {
            list: cli.check(Arg::flag("list"))?,
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
    Init,
    View,
    Read,
    Get,
    Tree,
    Lock,
    Build,
    Test,
    Launch,
    Search,
    Download,
    Install,
    Env,
    Config,
    Remove,
}

impl Topic {
    fn list_all() -> String {
        let list = [
            "new", "init", "view", "read", "get", "tree", "lock", "test", "build", "launch",
            "search", "download", "install", "env", "config", "remove",
        ];
        list.into_iter().fold(String::new(), |mut acc, x| {
            acc.push_str(&format!("{}\n", x));
            acc
        })
    }
}

impl std::str::FromStr for Topic {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "new" => Self::New,
            "init" => Self::Init,
            "view" => Self::View,
            "read" => Self::Read,
            "get" => Self::Get,
            "tree" => Self::Tree,
            "lock" => Self::Lock,
            "test" => Self::Test,
            "build" => Self::Build,
            "launch" => Self::Launch,
            "search" => Self::Search,
            "download" => Self::Download,
            "install" => Self::Install,
            "env" => Self::Env,
            "config" => Self::Config,
            "remove" => Self::Remove,
            _ => return Err(AnyError(format!("topic '{}' not found", s))),
        })
    }
}

impl Topic {
    /// Transforms the variant to its corresponding manual page.
    fn as_manual(&self) -> &str {
        use Topic::*;
        match &self {
            New => manuals::new::MANUAL,
            Init => manuals::init::MANUAL,
            View => manuals::view::MANUAL,
            Read => manuals::read::MANUAL,
            Get => manuals::get::MANUAL,
            Tree => manuals::tree::MANUAL,
            Lock => manuals::lock::MANUAL,
            Test => manuals::test::MANUAL,
            Build => manuals::build::MANUAL,
            Launch => manuals::launch::MANUAL,
            Search => manuals::search::MANUAL,
            Download => manuals::download::MANUAL,
            Install => manuals::install::MANUAL,
            Env => manuals::env::MANUAL,
            Config => manuals::config::MANUAL,
            Remove => manuals::remove::MANUAL,
        }
    }
}

impl Help {
    fn run(&self) -> Result<(), AnyError> {
        if self.list == true {
            println!("{}", Topic::list_all());
        } else {
            let contents = match &self.topic {
                Some(t) => t.as_manual(),
                None => manuals::orbit::MANUAL,
            };
            // TODO: check for a pager program to pipe contents into?
            println!("{}", contents);
        }
        Ok(())
    }
}

const HELP: &str = "\
Read in-depth documentation on Orbit topics.

Usage:
    orbit help [<topic>]

Args:
    <topic>         a listed topic or any orbit subcommand

Use 'orbit help --list' to see all available topics.
";
