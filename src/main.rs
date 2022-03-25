use orbit::cli;
use orbit::command::Command;
use orbit::command;

fn main() {
    match command::Orbit::load(&mut cli::Cli::new(std::env::args())) {
        Ok(cmd) => match cmd.run() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(101);
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(101);
        }
    }
}