#![allow(dead_code)]

mod interface;
mod commands;

pub mod seqalin;
pub mod pkgid;
mod cfgfile;
mod version;

use crate::interface::cli::*;
use crate::interface::errors::*;
use crate::interface::command::*;
use crate::commands::orbit::*;

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
                _ => eprintln!("error: {}", e)
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
            _ => eprintln!("error: {}", e),
        }
        return 101;
    }
    std::mem::drop(cli);
    // program level
    orbit.exec();
    0
}