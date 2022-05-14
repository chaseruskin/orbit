use crate::Command;
use crate::FromCli;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional};
use crate::interface::errors::CliError;
use crate::core::context::Context;
use crate::commands::manuals;
use crate::util::anyerror::AnyError;

#[derive(Debug, PartialEq)]
pub struct Help {
    topic: Option<Topic>,
}

#[derive(Debug, PartialEq)]
enum Topic {
    New,
    Plan,
    Build,
    Launch,
    Edit,
}

impl std::str::FromStr for Topic {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "new" => Self::New,
            "plan" => Self::Plan,
            "build" => Self::Build,
            "launch" => Self::Launch,
            "edit" => Self::Edit,
            _ => return Err(AnyError(format!("topic '{}' not found", s)))
        })
    }
}

impl Topic {
    /// Transforms the variant to its corresponding manual page.
    fn as_manual(&self) -> &str {
        use Topic::*;
        match &self {
            Edit => manuals::edit::MANUAL,
            New => manuals::new::MANUAL,
            Plan => manuals::plan::MANUAL,
            Build => manuals::build::MANUAL,
            Launch => manuals::launch::MANUAL,
        }
    }
}

impl Command for Help {
    type Err = Box<dyn std::error::Error>;
    fn exec(&self, _: &Context) -> Result<(), Self::Err> {
        self.run()?;
        Ok(())
    }
}

impl Help {
    fn run(&self) -> Result<(), AnyError> {
        let contents = match &self.topic {
            Some(t) => t.as_manual(),
            None => manuals::orbit::MANUAL
        };
        // @TODO check for a pager program to pipe contents into
        println!("{}", contents);
        Ok(())
    }
}

impl FromCli for Help {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(Help {
            topic: cli.check_positional(Positional::new("topic"))?,
        });
        command
    }
}

const HELP: &str = "\
Read in-depth documentation around Orbit topics.

Usage:
    orbit help [<topic>]

Args:
    <topic>         a listed topic or any orbit subcommand

Topics:
    cfg             learn about .cfg files
    cache           learn about orbit's caching system
    manifest        learn about the orbit.cfg file
    template        learn about templates
    blueprint       learn about generating a pre-build data file

Use 'orbit help --list' to see all available topics.
";