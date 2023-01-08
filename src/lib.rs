#![allow(dead_code)]

mod commands;
pub mod util;
mod core;

use clif::*;
use crate::commands::orbit::*;
use colored::*;
use clif::cmd::FromCli;
use clif::cmd::Command;
use crate::core::context::Context;

pub fn go() -> u8 {
    // interface level
    let mut cli = Cli::new()
        .emphasize_help()
        .threshold(2)
        .tokenize(std::env::args());

    let orbit = match Orbit::from_cli(&mut cli) {
        Ok(app) => {
            if let Err(err) = cli.is_empty() {
                match err.kind() {
                    ErrorKind::Help => println!("{}", err),
                    _ => eprintln!("{}: {}", "error".red().bold(), err)
                }
                return err.code()
            }
            std::mem::drop(cli);
            app
        },
        Err(err) => {
            match err.kind() {
                ErrorKind::Help => println!("{}", err),
                _ => eprintln!("{}: {}", "error".red().bold(), err)
            }
            return err.code()
        }
    };

    // program level
    match orbit.exec(&Context::new()) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e); 
            101
        }
    }
}