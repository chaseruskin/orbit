use std::io;
use std::io::{Error, Read};

pub fn prompt(s: &str) -> Result<bool, Error> {
    println!("{}? [y/n]", s);
    check_for_response(&mut io::stdin().lock())
}

/// Forever loops until a valid response is entered. "Y\n" and "\n" map to `true`, while
/// "N\n" maps to `false`.
fn check_for_response(input: &mut (impl Read + std::io::BufRead)) -> Result<bool, Error> {
    let mut buffer: String = String::new();
    loop {
        input.read_line(&mut buffer)?;
        let result = match buffer.to_uppercase().as_ref() {
            "\n" | "Y\n" => Some(true),
            "N\n" => Some(false),
            _ => {
                buffer.clear();
                None
            },
        };
        if let Some(r) = result {
            break Ok(r)
        };
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_input_to_output() {
        let r = check_for_response(&mut "n\n".as_bytes()).unwrap();
        assert_eq!(r, false);
        let r = check_for_response(&mut "N\n".as_bytes()).unwrap();
        assert_eq!(r, false);
        let r = check_for_response(&mut "\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "Y\n".as_bytes()).unwrap();
        assert_eq!(r, true);
        let r = check_for_response(&mut "y\n".as_bytes()).unwrap();
        assert_eq!(r, true);
    }
}