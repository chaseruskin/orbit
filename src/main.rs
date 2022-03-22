use orbit::cli;
use orbit::command::Command;
use orbit::command;

fn main() {
    let cli = cli::Cli::new(std::env::args());
    let cmd = command::Sum::initialize(cli);
    if let Ok(req) = cmd {
        req.run();
    } else {
        eprintln!("error: {}", cmd.unwrap_err());
    }
}