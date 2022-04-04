use std::env;

fn main() {
    println!("{}-{}", env::consts::ARCH, env::consts::OS);
}