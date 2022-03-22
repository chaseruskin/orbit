use orbit::cli;
use orbit::pkgid;

fn main() {
    let mut cli = cli::Cli::new(std::env::args());
    // first stage: identify command and general options
    let command = cli.next_positional::<String>();
    if command.is_err() {
        println!("orbit 0.1.0");
        return ();
    }
    cli.set_past(false);
    // second stage: match on command and assemble command
    let command = command.unwrap();
    match command.as_ref() {
        "new" => {
            let r = New {
                ip: cli.next_positional().unwrap(),
            };
            cli.is_clean().unwrap();

            if r.ip.fully_qualified().is_ok() {
                println!("IP is valid: {}", r.ip);
            } else {
                println!("IP {} is not fully qualified: {}", r.ip, r.ip.fully_qualified().unwrap_err());
            }
        }
        "complex" => {
            let r = Complex {
                code: cli.get_option(cli::Optional("--code")).unwrap(),
                level: cli.get_option(cli::Optional("--level")).unwrap().or(Some(99u8)),
                digits: cli.get_option_vec("--digit").unwrap(),
                en: cli.next_positional().unwrap(),
            };
            cli.is_clean().unwrap();
            println!("{:?}", r.level);
            println!("{:?}", r.code);
            println!("{:?}", r.en);
            if let Some(d) = r.digits {
                for i in d {
                    println!("{}", i);
                }
            }
        }
        _ => {
            println!("unknown subcommand: {}", command);
            return ()
        },
    }; 
}

struct New {
    ip: pkgid::PkgId,
}

struct Complex {
    en: bool,
    code: Option<String>,
    level: Option<u8>,
    digits: Option<Vec<u8>>,
}