use colored::*;

fn main() -> () {
    match install() {
        Ok(()) => (),
        Err(e) => eprintln!("{} {}", "error:".red().bold(), e)
    }
}

fn install() -> Result<(), Box<dyn std::error::Error>> {
    // route operating system accordingly
    if cfg!(unix) {
        unix()
    } else if cfg!(windows) {
        windows()
    } else {
        Err(InstallError::UnknownFamily)?
    }
}

use orbit::util::exepath::get_exe_path;
use orbit::util::prompt::prompt;

/// unix installation steps
fn unix() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    // gather path for what will be installed
    let contents = {
        let mut root = get_exe_path()?;
        // remove file to get parent directory
        root.pop();
        root.join("bin/orbit")
    };

    // 1. compute installation size
    let megabytes = fs_extra::dir::get_size(&contents)? as f32 / 1000000 as f32;
    println!("Installation size: {:.2} MB", megabytes);

    // 2. configure installation destination
    let path = PathBuf::from("/usr/local/bin");
    let path = installation_path(path)?;

    // 3. ask user for permission 
    match prompt("Install")? {
        true => {
            // 4a. copy the binary to the location
            std::fs::copy(contents, path.join("orbit"))?;
            println!("Successfully installed orbit");
        }
        false => {
            // 4b. cancel installation
            println!("Cancelled installation")
        }
    };
    Ok(())
}

/// windows installation steps
fn windows() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HEADER);

    // gather path for what will be installed
    let contents = {
        let mut root = get_exe_path()?;
        // remove file to get parent directory
        root.pop();
        root
    };

    // 1. compute installation size
    let megabytes = fs_extra::dir::get_size(&contents)? as f32 / 1000000 as f32;
    println!("Installation size: {:.2} MB", megabytes);

    // 2. configure installation destination
    let path = std::path::PathBuf::from(std::env::var("LOCALAPPDATA")?).join("Programs");
    let path = installation_path(path)?;

    // 3. ask user for permission
    match prompt("Install")? {
        true => {
            // 4a. copy the binary to the location
            let options = fs_extra::dir::CopyOptions::new();
            fs_extra::dir::copy(&contents, &path, &options)?;
            println!("Successfully installed orbit");

            println!("Tip: Add {} to the user PATH variable to call `orbit` from the command-line", path.join("orbit/bin").display())
        }
        false => {
            // 4b. cancel installation
            println!("Cancelled installation")
        }
    }
    Ok(())
}

use std::path::PathBuf;
use fs_extra;

fn installation_path(path: PathBuf) -> Result<PathBuf, Error> {
    println!("Default installation path: {}", path.display());
    let path = match set_variable("Enter installation path or press enter to continue:")? {
        Some(r) => PathBuf::from(r),
        None => path,
    };
    println!("Set installation path: {}", path.display());
    Ok(path)
}

#[derive(Debug, PartialEq)]
enum InstallError {
    UnknownFamily,
}

impl std::error::Error for InstallError {}

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

use std::io;

fn set_variable(msg: &str) -> Result<Option<String>, Error> {
    println!("{}", msg);
    let resp = capture_response(&mut io::stdin().lock())?;
    Ok(match resp.trim().is_empty() {
        true => None,
        false => Some(resp),
    })
}

use std::io::{Error, Read};

fn capture_response(input: &mut (impl Read + std::io::BufRead)) -> Result<String, Error> {
    let mut buffer: String = String::new();
    input.read_line(&mut buffer)?;
    Ok(buffer[0..buffer.len()-1].to_string())
}