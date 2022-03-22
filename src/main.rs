use orbit::cli;
use orbit::pkgid;

fn main() {
    let mut cli = cli::Cli::new(std::env::args());

    let command = cli.next_positional::<String>();
    if command.is_err() {
        println!("orbit 0.1.0");
        return ();
    }
    let command = command.unwrap();
    let runner = match command.as_ref() {
        "new" => New {
            ip: cli.next_positional().unwrap(),
        },
        _ => {
            println!("unknown subcommand: {}", command);
            return ()
        },
    };
    cli.is_clean().unwrap();

    if runner.ip.fully_qualified().is_ok() {
        println!("IP is valid: {}", runner.ip);
    } else {
        println!("IP {} is not fully qualified: {}", runner.ip, runner.ip.fully_qualified().unwrap_err());
    }
    
}

struct New {
    ip: pkgid::PkgId,
}