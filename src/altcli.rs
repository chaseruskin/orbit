use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Token {
    UnattachedArgument(usize, String),
    AttachedArgument(usize, String),
    Flag(usize, String),
    Switch(usize, char),
    Ignore(usize, String),
    Terminator(usize),
}

#[derive(Debug, PartialEq)]
struct Cli<'a> {
    tokens: Vec<Option<Token>>,
    flag_store: HashMap<&'a str, Vec<usize>>,
}

impl<'a> Cli<'a> {
    fn new() -> Self {
        Cli {
            tokens: Vec::new(),
            flag_store: HashMap::new(),
        }
    }

    fn tokenize<T: Iterator<Item=String>>(args: T) -> Self {
        let mut tokens = Vec::<Option<Token>>::new();
        let mut terminated = false;
        let mut args = args.skip(1).enumerate();
        while let Some((i, mut arg)) = args.next() {
            // ignore all input after detecting the terminator
            if terminated == true {
                tokens.push(Some(Token::Ignore(i, arg)));
            // handle an option
            } else if arg.starts_with('-') {
                // try to separate from '=' sign
                let mut value: Option<String> = None;
                let mut option: Option<String> = None;
                {
                    if let Some((opt, val)) = arg.split_once('=') {
                        option = Some(opt.to_string());
                        value = Some(val.to_string());
                    }
                }
                // update arg to be the value split by '='
                if let Some(opt) = option {
                    arg = opt;
                }
                // handle long flag signal
                if arg.starts_with("--") {
                    // caught the terminator (purely "--")
                    if arg.len() == 2 {
                        tokens.push(Some(Token::Terminator(i)));
                        terminated = true;
                    // caught a 'long option' flag
                    } else {
                        tokens.push(Some(Token::Flag(i, arg)));
                    }
                // handle short flag signal
                } else {
                    let mut arg = arg.chars().skip(1);
                    // split switches into individual components
                    while let Some(c) = arg.next() {
                        tokens.push(Some(Token::Switch(i, c)));
                    }
                }
                // caught an argument directly attached to an option
                if let Some(val) = value {
                    tokens.push(Some(Token::AttachedArgument(i, val)));
                }
            // caught an argument
            } else {
                tokens.push(Some(Token::UnattachedArgument(i, arg)));
            }
        }
        Cli { 
            tokens: tokens,
            flag_store: HashMap::new(),
        }
    }

    /// Takes out the next UnattachedArg from the token stream.
    fn next_uarg(&mut self) -> Option<String> {
        if let Some(p) = self.tokens
            .iter_mut()
            .find(|s| match s {
                Some(Token::UnattachedArgument(_, _)) => true,
                _ => false,
            }) {
                let q = p.take();
                if let Some(Token::UnattachedArgument(_, a)) = q {
                    Some(a)
                } else {
                    panic!("can only find unattached string!")
                }
        } else {
            None
        }
    }

    /// Returns all locations in the token stream where the flag is found.
    fn find_flag_positions(&self, s: &str) -> Option<&Vec<usize>> {
        Some(self.flag_store.get(s)?)
        // let locs: Vec<usize> = self.tokens.iter().enumerate().filter_map(|(i, t)| {
        //     match t {
        //         Token::Flag(_, id) => if id == s { Some(i) } else { None },
        //         _ => None,
        //     }
        // }).collect();
        // if locs.is_empty() {
        //     None
        // } else {
        //     Some(locs)
        // }
    }


    /// Grabs the flag from the token stream, and collects. If an argument were to follow
    /// it will be in the vector.
    fn pull_flag(&mut self, with_uarg: bool) -> Option<Vec<Option<String>>> {
        todo!()
    }


    /// Removes the ignored tokens from the stream, if they exist.
    fn get_reserved(&mut self) -> Option<Vec<String>> {
        todo!()
    }

    /// Returns all locations in the token stream where the flag is found.
    fn find_switch_positions(&self, c: &char) -> Option<Vec<usize>> {
        let locs: Vec<usize> = self.tokens.iter().enumerate().filter_map(|(i, t)| {
            match t {
                Some(Token::Switch(_, id)) => if id == c { Some(i) } else { None },
                _ => None,
            }
        }).collect();
        if locs.is_empty() {
            None
        } else {
            Some(locs)
        }
    }

    fn stash_flags(tokens: &Vec<Option<Token>>) -> HashMap<&str, Vec<usize>> {
        let mut store = HashMap::new();
        tokens.iter().enumerate().for_each(|(i, t)| {
            match t {
                Some(Token::Flag(_, id)) => {
                    store.entry(id.as_ref()).or_insert(Vec::new()).push(i);
                }
                _ => (),
            }
        });
        store
    }
}


#[cfg(test)]
mod test {
    use super::*;

    /// Helper test fn to write vec of &str as iterator for Cli parameter.
    fn args<'a>(args: Vec<&'a str>) -> Box<dyn Iterator<Item=String> + 'a> {
        Box::new(args.into_iter().map(|f| f.to_string()).into_iter())
    }

    fn sample_cli1<'a>() -> Cli<'a> {
        Cli { tokens: vec![
            Some(Token::Flag(0, "--help".to_string())),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4, "--lib".to_string())),
            Some(Token::Flag(5, "--name".to_string())),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(5, "--help".to_string())),
            Some(Token::Switch(6, 's')),
            Some(Token::Switch(6, 'c')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Terminator(7)),
            Some(Token::Ignore(8, "--map".to_string())),
            Some(Token::Ignore(9, "synthesis".to_string())),
            Some(Token::Ignore(10, "-jto".to_string())),
            ],
            flag_store: HashMap::new(),
        }
    }

    #[test]
    fn tokenizer() {
        let cli = Cli::tokenize(args(vec![]));
        assert_eq!(cli.tokens, vec![]);

        let cli = Cli::tokenize(args(vec!["orbit"]));
        assert_eq!(cli.tokens, vec![]);

        let cli = Cli::tokenize(args(vec!["orbit", "--help"]));
        assert_eq!(cli.tokens, vec!
            [Some(Token::Flag(0, "--help".to_string()))
            ]
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-v"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0, "--help".to_string())), 
            Some(Token::Switch(1, 'v'))
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "new", "rary.gates"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::UnattachedArgument(0, "new".to_string())), 
            Some(Token::UnattachedArgument(1, "rary.gates".to_string())),
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-vh"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0, "--help".to_string())), 
            Some(Token::Switch(1, 'v')),
            Some(Token::Switch(1, 'h')),
            ],
        );

        let cli = Cli::tokenize(args(vec!["orbit", "--help", "-vhc=10"]));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0, "--help".to_string())), 
            Some(Token::Switch(1, 'v')),
            Some(Token::Switch(1, 'h')),
            Some(Token::Switch(1, 'c')),
            Some(Token::AttachedArgument(1, "10".to_string())),
            ],
        );

        // final boss
        let cli = Cli::tokenize(args(
            vec!["orbit", "--help", "-v", "new", "ip", "--lib", "--name=rary.gates", "--help", "-sci", "--", "--map", "synthesis", "-jto"]
        ));
        assert_eq!(cli.tokens, vec![
            Some(Token::Flag(0, "--help".to_string())),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4, "--lib".to_string())),
            Some(Token::Flag(5, "--name".to_string())),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(6, "--help".to_string())),
            Some(Token::Switch(7, 's')),
            Some(Token::Switch(7, 'c')),
            Some(Token::Switch(7, 'i')),
            Some(Token::Terminator(8)),
            Some(Token::Ignore(9, "--map".to_string())),
            Some(Token::Ignore(10, "synthesis".to_string())),
            Some(Token::Ignore(11, "-jto".to_string())),
            ],
        );
    }

    #[test]
    fn find_flags_and_switches() {
        let mut cli = Cli { tokens: vec![
            Some(Token::Flag(0, "--help".to_string())),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4, "--lib".to_string())),
            Some(Token::Flag(5, "--name".to_string())),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(5, "--help".to_string())),
            Some(Token::Switch(6, 's')),
            Some(Token::Switch(6, 'c')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Terminator(7)),
            Some(Token::Ignore(8, "--map".to_string())),
            Some(Token::Ignore(9, "synthesis".to_string())),
            Some(Token::Ignore(10, "-jto".to_string())),
            ],
            flag_store: HashMap::new(),
        };
        cli.flag_store = Cli::stash_flags(&cli.tokens); 

        // detects 0
        assert_eq!(cli.find_flag_positions("--version"), None);
        // detects 1
        assert_eq!(cli.find_flag_positions("--lib"), Some(&vec![4]));
        // detects multiple
        assert_eq!(cli.find_flag_positions("--help"), Some(&vec![0, 7]));
        // flag was past terminator and marked as ignore
        assert_eq!(cli.find_flag_positions("--map"), None);
        // filters out arguments
        assert_eq!(cli.find_flag_positions("rary.gates"), None);

        // detects 0
        assert_eq!(cli.find_switch_positions(&'q'), None);
        // detects 1
        assert_eq!(cli.find_switch_positions(&'v'), Some(vec![1]));
        // detects multiple
        assert_eq!(cli.find_switch_positions(&'i'), Some(vec![10, 11]));
        // switch was past terminator and marked as ignore
        assert_eq!(cli.find_switch_positions(&'j'), None);
    }

    #[test]
    fn flags_in_map() {
        let tokens = vec![
            Some(Token::Flag(0, "--help".to_string())),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4, "--lib".to_string())),
            Some(Token::Flag(5, "--name".to_string())),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(5, "--help".to_string())),
            Some(Token::Switch(6, 's')),
            Some(Token::Switch(6, 'c')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Terminator(7)),
            Some(Token::Ignore(8, "--map".to_string())),
            Some(Token::Ignore(9, "synthesis".to_string())),
            Some(Token::Ignore(10, "-jto".to_string())),
        ];

        let store = Cli::stash_flags(&tokens);
        let mut opt_store = HashMap::<&str, Vec<usize>>::new();
        opt_store.insert(&"--help", vec![0, 7]);
        opt_store.insert(&"--lib", vec![4]);
        opt_store.insert(&"--name", vec![5]);
        assert_eq!(store, opt_store);
    }

    #[test]
    fn take_unattached_args() {
        let mut cli = Cli { tokens: vec![
            Some(Token::Flag(0, "--help".to_string())),
            Some(Token::Switch(1, 'v')),
            Some(Token::UnattachedArgument(2, "new".to_string())), 
            Some(Token::UnattachedArgument(3, "ip".to_string())),
            Some(Token::Flag(4, "--lib".to_string())),
            Some(Token::Flag(5, "--name".to_string())),
            Some(Token::AttachedArgument(5, "rary.gates".to_string())),
            Some(Token::Flag(5, "--help".to_string())),
            Some(Token::Switch(6, 's')),
            Some(Token::Switch(6, 'c')),
            Some(Token::Switch(6, 'i')),
            Some(Token::Switch(6, 'i')),
            Some(Token::UnattachedArgument(7, "get".to_string())),
            Some(Token::Terminator(8)),
            Some(Token::Ignore(9, "--map".to_string())),
            Some(Token::Ignore(10, "synthesis".to_string())),
            Some(Token::Ignore(11, "-jto".to_string())),
            ],
            flag_store: HashMap::new(),
        };

        assert_eq!(cli.next_uarg().unwrap(), "new".to_string());
        assert_eq!(cli.next_uarg().unwrap(), "ip".to_string());
        assert_eq!(cli.next_uarg().unwrap(), "get".to_string());
        assert_eq!(cli.next_uarg(), None);
    }
}

// orbit --help -v new ip --lib --name=rary.gates -sci -- --map synthesis

// noop Flag Switch Arg Arg Flag Flag Arg Switch Switch Switch Flag Ignore Ignore

// grammar, collect tokens
/*

Subcommand -> 


*/

/*
Build a command (ideal)

struct DoSomething {
    action: String,
    repeat: Option<u8>,
}


fn new() -> Self {
    DoSomething {
        action: args.match(Positional::new("action"))
    }
}


*/
