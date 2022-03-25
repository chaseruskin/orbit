use orbit::cli;
use orbit::command::Command;
use orbit::command;

fn main() {
    let cmd = command::Orbit::load(
        &mut cli::Cli::new(std::env::args())
    );
    if let Ok(req) = cmd {
        req.run();
    } else {
        eprintln!("error: {}", cmd.unwrap_err());
        std::process::exit(101);
    }
}