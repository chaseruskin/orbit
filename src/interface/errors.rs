use crate::interface::arg::Arg;
use std::fmt::Display;
use std::error::Error;
use colored::*;

#[derive(Debug, PartialEq)]
pub enum CliError<'a> {
    BadType(Arg<'a>, String),
    MissingPositional(Arg<'a>, String),
    DuplicateOptions(Arg<'a>),
    ExpectingValue(Arg<'a>),
    UnexpectedValue(Arg<'a>, String),
    OutOfContextArg(String),
    OutOfContextArgSuggest(String, String),
    UnexpectedArg(String),
    SuggestArg(String, String),
    SuggestSubcommand(String, String),
    UnknownSubcommand(Arg<'a>, String),
    BrokenRule(String),
    Help(&'a str),
}

impl<'a> Error for CliError<'a> {}

impl<'a> Display for CliError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        use CliError::*;
        let footer = format!("\n\nFor more information try {}", "--help".green());
        match self {
            Help(h) => write!(f, "{}", h),
            SuggestArg(a, sug) => write!(f, "unknown argument '{}'\n\nDid you mean '{}'?", a.yellow(), sug.green()),
            SuggestSubcommand(a, sug) => write!(f, "unknown subcommand '{}'\n\nDid you mean '{}'?", a.yellow(), sug.green()),
            OutOfContextArgSuggest(o, cmd) => write!(f, "argument '{}' is unknown, or invalid in the current context\n\nMaybe move it after '{}'?{}", o.yellow(), cmd.green(), footer),
            OutOfContextArg(o) => write!(f, "argument '{}' is unknown, or invalid in the current context{}", o.yellow(), footer),
            BadType(a, e) => write!(f, "argument '{}' did not process due to {}{}", a, e, footer),
            MissingPositional(p, u) => {
                // detect the usage statement to print from command's short help text
                let usage = if let Some(text) = u.split_terminator('\n').skip(3).next() {
                    "usage:\n".to_string() + text
                } else {
                    "".to_string()
                };
                write!(f, "missing required argument '{}'\n{}{}", p, usage, footer)
            },
            DuplicateOptions(o) => write!(f, "option '{}' was requested more than once, but can only be supplied once{}", o.to_string().yellow(), footer),
            ExpectingValue(x) => write!(f, "option '{}' expects a value but none was supplied{}", x, footer),
            UnexpectedValue(x, s) => write!(f, "flag '{}' cannot accept values but one was supplied \"{}\"{}", x, s, footer),
            UnexpectedArg(s) => write!(f, "unknown argument '{}'{}", s.yellow(), footer),
            UnknownSubcommand(c, a) => write!(f, "'{}' is not a valid subcommand for {}{}", a, c, footer),
            BrokenRule(r) => write!(f, "a rule conflict occurred from {}", r),
        }
    }
}