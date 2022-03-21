use orbit::cli;
use orbit::pkgid;

fn main() {
    let mut cli = cli::Cli::new(std::env::args());
    println!("orbit 0.1.0");

    let command: String = cli.next_positional().unwrap();
    println!("orbit-info: requesting command- {}", command);
    let runner = New {
        ip: cli.next_positional().unwrap(),
    };
    if runner.ip.fully_qualified() {
        println!("orbit-info: ip valid- {}", runner.ip);
    } else {
        println!("orbit-error: ip is not fully qualified- {}", runner.ip);
    }
    
}

struct New {
    ip: pkgid::PkgId,
}