#![allow(dead_code)]

mod interface;
mod commands;
mod util;
mod core;

use crate::interface::cli::*;
use crate::interface::errors::*;
use crate::interface::command::*;
use crate::commands::orbit::*;
use colored::*;

pub fn run() -> u8 {
    // interface level
    let mut cli = Cli::tokenize(std::env::args());
    let orbit = match Orbit::from_cli(&mut cli) {
        Ok(r) => r,
        Err(e) => {
            match e {
                CliError::Help(s) => {
                    println!("{}", s);
                    return 0;
                }
                _ => eprintln!("{} {}", "error:".red().bold(), e)
            }
            return 101;
        }
    };
    if let Err(e) = cli.is_empty() {
        match e {
            CliError::Help(s) => {
                println!("{}", s);
                return 0;
            }
            _ => eprintln!("{} {}", "error:".red().bold(), e),
        }
        return 101;
    }
    std::mem::drop(cli);
    // program level
    orbit.exec();
    0
}