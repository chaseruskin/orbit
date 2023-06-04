use crate::util::anyerror::Fault;
use std::ffi::OsStr;

#[derive(Debug, PartialEq)]
struct Statement(Vec<Argument>);

#[derive(Debug, PartialEq)]
enum Argument {
    Quoted(String),
    Bare(String),
}

impl AsRef<OsStr> for Argument {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Bare(s) => s.as_ref(),
            Self::Quoted(s) => s.as_ref(),
        }
    }
}

impl AsRef<str> for Argument {
    fn as_ref(&self) -> &str {
        match self {
            Self::Bare(s) => s.as_ref(),
            Self::Quoted(s) => s.as_ref(),
        }
    }
}

impl Argument {
    fn as_string(&self) -> &String {
        match self {
            Self::Bare(s) => s,
            Self::Quoted(s) => s,
        }
    }
}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quoted(s) => write!(f, "\"{}\"", s),
            Self::Bare(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for Statement {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        let mut chars = s.chars();
        let mut in_quote = false;
        let mut buffer = String::new();
        while let Some(c) = chars.next() {
            match c {
                '\'' | '\"' => {
                    in_quote = !in_quote;
                    if in_quote == false {
                        if buffer.is_empty() == false {
                            result.push(Argument::Quoted(buffer.clone()));
                        }
                        buffer.clear();
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    if in_quote == false {
                        if buffer.is_empty() == false {
                            result.push(Argument::Bare(buffer.clone()));
                        }
                        buffer.clear();
                    } else {
                        buffer.push(c);
                    }
                }
                _ => buffer.push(c),
            }
        }
        if buffer.is_empty() == false {
            result.push(Argument::Bare(buffer));
        }
        Ok(Self(result))
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .fold(String::new(), |acc, x| acc + " " + x.as_ref())
        )
    }
}

impl Statement {
    fn execute(&self) -> Result<(), Fault> {
        println!("hook:{}", self);
        let proc = std::process::Command::new(&self.0.first().unwrap().as_string())
            .args(&self.0.as_slice()[1..])
            .output()?;

        match proc.status.code() {
            Some(num) => {
                if num != 0 {
                    panic!("error code")
                } else {
                    ()
                }
            }
            None => panic!("sig termination"),
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Hook(Vec<Statement>);

impl std::str::FromStr for Hook {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_terminator('\n')
                .filter_map(|f| {
                    if f.is_empty() {
                        None
                    } else {
                        Some(f.parse::<Statement>().unwrap())
                    }
                })
                .collect(),
        ))
    }
}

impl Hook {
    pub fn execute(&self) -> Result<(), Fault> {
        for stmt in &self.0 {
            stmt.execute()?
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn it_works() {
        let text = "a b c\n\nd\te  f\r\ng h";
        let hook = Hook::from_str(text).unwrap();
        assert_eq!(
            hook.0,
            vec![
                Statement(vec![
                    Argument::Bare(String::from("a")),
                    Argument::Bare(String::from("b")),
                    Argument::Bare(String::from("c"))
                ]),
                Statement(vec![
                    Argument::Bare(String::from("d")),
                    Argument::Bare(String::from("e")),
                    Argument::Bare(String::from("f"))
                ]),
                Statement(vec![
                    Argument::Bare(String::from("g")),
                    Argument::Bare(String::from("h"))
                ]),
            ]
        );
    }

    #[test]
    fn statement_from_str() {
        let text = "git commit -m \"my message\"";
        assert_eq!(
            Statement::from_str(text)
                .unwrap()
                .0
                .into_iter()
                .map(|f| f.as_string().to_string())
                .collect::<Vec<String>>(),
            vec!["git", "commit", "-m", "my message"]
                .into_iter()
                .map(|f| f.to_owned())
                .collect::<Vec<String>>()
        );
    }
}
