use colored::*;

fn main() -> () {
    match install() {
        Ok(()) => (),
        Err(e) => eprintln!("{} {}", "error:".red().bold(), e)
    }
}

fn install() -> Result<(), InstallError> {
    // route operating system accordingly
    if cfg!(unix) {
        unix()
    } else if cfg!(windows) {
        windows()
    } else {
        Err(InstallError::UnknownFamily)
    }
}

/// unix installation steps
fn unix() -> Result<(), InstallError> {
    println!("{}", HEADER);
    Ok(())
}

/// windows installation steps
fn windows() -> Result<(), InstallError> {
    println!("{}", HEADER);
    Ok(())
}


#[derive(Debug, PartialEq)]
enum InstallError {
    UnknownFamily,
}

use std::fmt::Display;

impl Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        match self {
            Self::UnknownFamily => write!(f, "unknown family (did not detect unix or windows)")
        }
    }
}

const HEADER: &str = "\
------------------------------------------------------------
::              ORBIT INSTALLATION PROGRAM                ::
------------------------------------------------------------
";