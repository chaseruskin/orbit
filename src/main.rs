use cliproc::*;
use orbit::Orbit;
use std::env;

fn main() -> ExitCode {
    Cli::default().parse(env::args()).go::<Orbit>()
}
