use std::io::BufReader;
use std::io::Read as ReadTrait;
use std::io::Write;
use std::path::PathBuf;

use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::lexer::Position;
use crate::core::lang::vhdl::token::Identifier;
use crate::core::lang::vhdl::token::VHDLToken;
use crate::core::lang::vhdl::token::VHDLTokenizer;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::sha256;
use crate::OrbitResult;
use clif::arg::{Flag, Optional, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use crate::core::lang::lexer::Token;

use super::get::GetError;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Read {
    unit: Identifier,
    ip: Option<PartialIpSpec>,
    location: bool,
    file: bool,
    keep: bool,
    reverse: bool,
    start: Option<VHDLTokenizer>,
    end: Option<VHDLTokenizer>,
    follow: Option<VHDLTokenizer>,
    limit: Option<usize>,
}

impl FromCli for Read {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(HELP).ref_usage(2..4))?;
        let command = Ok(Read {
            // flags
            file: cli.check_flag(Flag::new("file"))?,
            location: cli.check_flag(Flag::new("location"))?,
            keep: cli.check_flag(Flag::new("keep"))?,
            reverse: cli.check_flag(Flag::new("reverse"))?,
            // options
            limit: cli.check_option(Optional::new("limit"))?,
            ip: cli.check_option(Optional::new("ip").value("spec"))?,
            start: cli.check_option(Optional::new("start").value("code"))?,
            end: cli.check_option(Optional::new("end").value("code"))?,
            follow: cli.check_option(Optional::new("follow").value("code"))?,
            // positionals
            unit: cli.require_positional(Positional::new("unit"))?,
        });
        command
    }
}

impl Command<Context> for Read {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // determine the destination
        let dest: PathBuf = c.get_home_path().join(TMP_DIR);

        // attempt to clean the tmp directory when --keep is disabled
        if dest.exists() == true && self.keep == false {
            // do not error if this procedure fails
            match std::fs::remove_dir_all(&dest) {
                Ok(_) => (),
                // @todo: warn user that directory was unable to be cleaned
                Err(_) => (),
            }
        }
        // cast the destination folder into an option based on if the user wants a file
        let dest = match self.file {
            true => Some(dest),
            false => None,
        };

        // checking external IP
        if let Some(tg) = &self.ip {
            // gather the catalog (all manifests)
            let catalog = Catalog::new().installations(c.get_cache_path())?;

            // access the requested ip
            match catalog.inner().get(&tg.get_name()) {
                Some(lvl) => {
                    let inst = match lvl.get_install(tg.get_version()) {
                        Some(i) => i,
                        None => panic!("version does not exist for this ip"),
                    };
                    self.run(inst, dest.as_ref())
                }
                None => {
                    // the ip does not exist
                    return Err(AnyError(format!("Failed to find IP {}", tg)))?
                }
            }
        // must be in an IP if omitting the pkgid
        } else {
            let ip = match c.get_ip_path() {
                Some(p) => Ip::load(p.to_path_buf())?,
                None => return Err(AnyError(format!("Not within an existing ip")))?,
            };

            self.run(&ip, dest.as_ref())
        }
    }
}

impl Read {
    fn run(&self, target: &Ip, dest: Option<&PathBuf>) -> Result<(), Fault> {
        let (path, loc) = Self::read(&self.unit, &target, dest)?;

        let path = {
            if self.location == true {
                PathBuf::from({
                    let mut p = path.as_os_str().to_os_string();
                    p.push(&loc.to_string());
                    p
                })
            } else {
                path
            }
        };

        // dump the file contents of the source code to the console if there was no destination
        let print_to_console = dest.is_none();

        // access the tokens
        let contents = fs::read_to_string(&path)?;
        let src_tokens = VHDLTokenizer::from_source_code(&contents).into_tokens_all();

        // perform a search on tokens
        let (start, follow,end) = {
            // get the tokens
            let start_tokens = match &self.start {
                Some(tokenizer) => tokenizer.as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VHDLToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let end_tokens = match &self.end {
                Some(tokenizer) => tokenizer.as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VHDLToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let follow_tokens = match &self.follow {
                Some(tokenizer) => tokenizer.as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VHDLToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            // search over the source code tokens
            let start = Self::find_location(&src_tokens, &start_tokens);

            let remaining_tokens = match &start {
                Some(pos) => {
                    match self.reverse {
                        true => src_tokens.into_iter().rev().skip_while(|p| p.locate() >= pos).collect(),
                        false => src_tokens.into_iter().skip_while(|p| p.locate() <= pos).collect(),
                    }
                },
                None => match self.reverse {
                    true => src_tokens.into_iter().rev().collect(),
                    false => src_tokens,
                }
            };
            
            let follow_tokens = match self.reverse {
                true => follow_tokens.into_iter().rev().collect(),
                false => follow_tokens,
            };

            let follow = Self::find_location(&remaining_tokens, &follow_tokens);

            let end_tokens = match self.reverse {
                true => end_tokens.into_iter().rev().collect(),
                false => end_tokens,
            };
            let end = Self::find_location(&remaining_tokens, &end_tokens);

            (start, follow, end)
        };

        // sort among all three items
        let mut points: [&Option<Position>; 3] = [&start, &follow, &end];
        points.sort();
        let points: Vec<&Option<Position>> = points.into_iter().filter(|p| p.is_some() == true).collect();
        // println!("{:?}", points);

        let segment: String = {
            let iter = contents.split_terminator('\n');

            let iter = match &points.get(0) {
                Some(p) => iter.skip(p.as_ref().unwrap().line()-1),
                None => iter.skip(0),
            };
            let iter = match &points.get(1) {
                Some(p) => iter.take(p.as_ref().unwrap().line()-points.get(0).unwrap().as_ref().unwrap_or(&Position::new()).line()+1),
                None => iter.take(usize::MAX),
            };
            let iter = iter.map(|line| line.to_string() + "\n");

            match self.limit {
                Some(l) => iter.take(l).collect(),
                None => iter.collect(),
            }
        };

        println!(
            "{}",
            match print_to_console {
                // display the contents
                true => { segment },
                // overwrite contents and display the file path
                false => {
                    let cut_code = start.is_some() || end.is_some() || follow.is_some();

                    let file = match cut_code {
                        true => {
                            // create and write a temporary file
                            let mut file = std::fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(&path)?;

                            file.write(&segment.as_bytes())?;
                            file.flush()?;
                            file
                        }, 
                        false => {
                            std::fs::OpenOptions::new().read(true).open(&path)?
                        },
                    };

                    // set to read-only
                    let mut perms = file.metadata()?.permissions();
                    perms.set_readonly(true);
                    file.set_permissions(perms)?;
                    path.display().to_string()
                }
            }
        );
        Ok(())
    }

    fn check_tokens_eq(source: &VHDLToken, sub: &VHDLToken) -> bool {
        match sub {
            // skip EOF token
            &VHDLToken::EOF => { true },
            // only match on the fact that they are comments
            VHDLToken::Comment(_) => { source.as_comment().is_some() },
            _ => { source == sub },
        }
    }

    fn find_location(src_tokens: &Vec<Token<VHDLToken>>, find_tokens: &Vec<&Token<VHDLToken>>) -> Option<Position> {
        let mut tracking: bool;
        let mut src_tokens_iter = src_tokens.iter();
        while let Some(t) = src_tokens_iter.next() {
            // begin to see if we start tracking
            // println!("{:?} {:?}", find_tokens.first().unwrap().as_type(), t.as_type());

            if find_tokens.len() > 0 && Self::check_tokens_eq(t.as_type(), find_tokens.first().unwrap().as_type()) == true {
                // println!("{}", "HERE");
                let mut find_tokens_iter = find_tokens.iter().skip(1);
                tracking = true;
                while let Some(find_t) = find_tokens_iter.next() {
                    // skip the EOF token
                    if find_t.as_type() == &VHDLToken::EOF { continue; }
                    if let Some(source_t) = src_tokens_iter.next() {
                        // lost sight
                        if Self::check_tokens_eq(source_t.as_type(), find_t.as_type()) == false {
                            tracking = false;
                            break;
                        }
                    } else {
                        tracking = false;
                        break;
                    }
                }
                // initiate lock
                if tracking == true {
                    // @todo: return last index of list for skipping purposes
                    return Some(t.locate().clone());
                }
            }
            // @todo: handle follow better (keep iterating until hitting a wrong one)
        }
        None
    }

    /// Finds the filepath and file position for the provided primary design unit `unit`
    /// under the project `ip`.
    ///
    /// If `dest` contains a value, it will create a new directory at `dest` and copy
    /// the file to be read-only. If it is set to `None`, then it will open the
    /// file it is referencing (no copy).
    fn read(
        unit: &Identifier,
        ip: &Ip,
        dest: Option<&PathBuf>,
    ) -> Result<(PathBuf, Position), Fault> {
        // find the unit
        let units = Ip::collect_units(true, ip.get_root())?;

        // get the file data for the primary design unit
        let (source, position) = match units.get_key_value(unit) {
            Some((_, unit)) => (
                unit.get_unit().get_source_code_file(),
                unit.get_unit().get_symbol().unwrap().get_position().clone(),
            ),
            None => {
                return Err(GetError::SuggestShow(
                    GetError::EntityNotFound(
                        unit.clone(),
                        ip.get_man().get_ip().get_name().clone(),
                        ip.get_man().get_ip().get_version().clone(),
                    )
                    .to_string(),
                    ip.get_man().get_ip().get_name().clone(),
                    ip.get_man().get_ip().get_version().clone(),
                ))?
            }
        };

        let (checksum, bytes) = {
            // open the file and create a checksum
            let mut bytes = Vec::new();
            let file = std::fs::File::open(source)?;
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut bytes)?;
            (sha256::compute_sha256(&bytes), bytes)
        };

        let src = PathBuf::from(source);

        match dest {
            // return direct reference if no `dest` (within current ip)
            None => return Ok((src, position)),
            Some(dest) => {
                // create new file under checksum directory
                let dest = dest.join(&checksum.to_string().get(0..10).unwrap());
                std::fs::create_dir_all(&dest)?;

                // add filename to destination path
                let dest = dest.join(src.file_name().unwrap());

                // try to remove file if it exists
                if dest.exists() == false || std::fs::remove_file(&dest).is_ok() {
                    // create and write a temporary file
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(&dest)?;
                    file.write(&bytes)?;
                    file.flush()?;
                }
                Ok((dest, position))
            }
        }
    }
}

const TMP_DIR: &str = "tmp";

const HELP: &str = "\
Inspect hdl design unit source code.

Usage:
    orbit read [options] <unit>

Args:
    <unit>                  primary design unit identifier

Options:            
    --ip <spec>             ip to reference the unit from
    --location              append the :line:col to the filepath
    --file                  display the path to the read-only source code
    --keep                  prevent previous files read from being deleted
    --limit <num>           set a maximum number of lines to print

Use 'orbit help read' to learn more about the command.
";
