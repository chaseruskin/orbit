use crate::interface::cli::Cli;
use std::fmt::Debug;
use crate::interface::errors::CliError;

pub trait Command: Debug {
    fn exec(&self) -> ();
}

pub trait FromCli {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError<'c>> where Self: Sized;
}

pub trait Runner: Command + FromCli + Debug {}

#[cfg(test)]
mod test {
    use crate::interface::arg::*;
    use super::*;

    /// Helper test fn to write vec of &str as iterator for Cli parameter.
    fn args<'a>(args: Vec<&'a str>) -> Box<dyn Iterator<Item=String> + 'a> {
        Box::new(args.into_iter().map(|f| f.to_string()).into_iter())
    }

    #[derive(Debug, PartialEq)]
    struct Add {
        lhs: u32,
        rhs: u32,
        verbose: bool,
    }

    impl Command for Add {
        fn exec(&self) -> () {
            println!("{}", self.run());
        }
    }

    impl Add {
        /// Simple fn to return an answer for the `Add` test command.
        fn run(&self) -> String {
            let sum = self.lhs + self.rhs;
            match self.verbose {
                true => format!("{} + {} = {}", self.lhs, self.rhs, sum),
                false => format!("{}", sum),
            }
        }
    }

    impl FromCli for Add {
        fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
            // the ability to "learn options" beforehand is possible, or can be skipped
            // "learn options" here (take in known args (as ref?))
            // -- possible CLI API --
            // opti_all -> accept multiple values (must have >= 1, all Non-none, all valid) (Option<Vec<T>>)
            // flag_cnt -> count number of flag raises (usize)
            // flag_one -> verify only one flag raised if any (bool)
            // opti_one -> accept only 1 valid value parsed from str (Option<T>)
            // position -> get next argument (T)
            // ...
            //  ip: value(Positional("ip"))?.require()?
            //  name: value(Optional("count"))?.default(10)?
            // ...
            // nest_cmd -> gather subcommand, and then recursively create command using `from_cli` to store as dyn Command
            // note: overload `fn drop` for cli to check itself if it is clean (no unhandled args)
            // --
            // :todo: add usage of cli (API..?)
            Ok(Add {
                // then call API cli.query(Arg::Flag("verbose"))?
                verbose: cli.check_flag(Flag::new("verbose"))?,
                lhs: cli.require_positional(Positional::new("lhs"))?,
                rhs: cli.require_positional(Positional::new("rhs"))?,
            })
        }
    }

    // testing a nested subcommand cli structure
    #[derive(Debug)]
    struct Op {
        command: Box<dyn Command>,
    }

    impl Command for Op {
        fn exec(&self) -> () {
            self.command.exec();
        }
    }

    impl FromCli for Op {
        fn from_cli<'c>(cli: &'c mut Cli<'_>) -> Result<Self, CliError<'c>> { 
            // Ok(Op {
            //     command: // ??
            // });
            todo!()
        }
    }

    enum OpSubcommand {
        Add(Add)
    }


    #[test]
    fn make_add_command() {
        let mut cli = Cli::tokenize(args(vec!["add", "9", "10"]));
        let add = Add::from_cli(&mut cli).unwrap();
        assert_eq!(add, Add {
            lhs: 9,
            rhs: 10,
            verbose: false
        });

        let mut cli = Cli::tokenize(args(vec!["add", "1", "4", "--verbose"]));
        let add = Add::from_cli(&mut cli).unwrap();
        assert_eq!(add, Add {
            lhs: 1,
            rhs: 4,
            verbose: true
        });

        let mut cli = Cli::tokenize(args(vec!["add", "5", "--verbose", "2"]));
        let add = Add::from_cli(&mut cli).unwrap();
        assert_eq!(add, Add {
            lhs: 5,
            rhs: 2,
            verbose: true
        });
    }
}