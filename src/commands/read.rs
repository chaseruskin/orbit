use std::io::BufReader;
use std::io::Read as ReadTrait;
use std::io::Write;
use std::path::PathBuf;

use super::get::GetError;
use crate::commands::helps::read;
use crate::core::catalog::Catalog;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::ip::PartialIpSpec;
use crate::core::lang::lexer::Position;
use crate::core::lang::lexer::Token;
use crate::core::lang::vhdl::token::VhdlToken;
use crate::core::lang::vhdl::token::VhdlTokenizer;
use crate::core::lang::LangIdentifier;
use crate::core::lang::LangMode;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::util::sha256;
use crate::OrbitResult;
use clif::arg::{Flag, Optional, Positional};
use clif::cmd::{Command, FromCli};
use clif::Cli;
use clif::Error as CliError;
use std::fs;

const TMP_DIR: &str = "tmp";

#[derive(Debug, PartialEq)]
pub struct Read {
    unit: LangIdentifier,
    ip: Option<PartialIpSpec>,
    location: bool,
    file: bool,
    keep: bool,
    start: Option<VhdlTokenizer>,
    end: Option<VhdlTokenizer>,
    comment: Option<VhdlTokenizer>,
    limit: Option<usize>,
}

impl FromCli for Read {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self, CliError> {
        cli.check_help(clif::Help::new().quick_text(read::HELP).ref_usage(2..4))?;
        let command = Ok(Read {
            // flags
            file: cli.check_flag(Flag::new("file"))?,
            location: cli.check_flag(Flag::new("location"))?,
            keep: cli.check_flag(Flag::new("keep"))?,
            // options
            limit: cli.check_option(Optional::new("limit").value("num"))?,
            ip: cli.check_option(Optional::new("ip").value("spec"))?,
            start: cli.check_option(Optional::new("start").value("code"))?,
            end: cli.check_option(Optional::new("end").value("code"))?,
            comment: cli.check_option(Optional::new("doc").value("code"))?,
            // positionals
            unit: cli.require_positional(Positional::new("unit"))?,
        });
        command
    }
}

impl Command<Context> for Read {
    type Status = OrbitResult;

    fn exec(&self, c: &Context) -> Self::Status {
        // verify location is only set iff the file mode is enabled
        if self.location == true && self.file == false {
            Err(AnyError(format!(
                "The flag '--location' can only be set when using '--file'"
            )))?
        }

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
                    self.run(inst, dest.as_ref(), &c.get_lang_mode())
                }
                None => {
                    // the ip does not exist
                    return Err(AnyError(format!("Failed to find ip {}", tg)))?;
                }
            }
        // must be in an IP if omitting the pkgid
        } else {
            let ip = match c.get_ip_path() {
                Some(p) => Ip::load(p.to_path_buf(), true)?,
                None => return Err(AnyError(format!("Not within an existing ip")))?,
            };

            self.run(&ip, dest.as_ref(), &c.get_lang_mode())
        }
    }
}

impl Read {
    fn run(&self, target: &Ip, dest: Option<&PathBuf>, mode: &LangMode) -> Result<(), Fault> {
        let (path, loc) = Self::read(&self.unit, &target, dest, mode)?;

        // dump the file contents of the source code to the console if there was no destination
        let print_to_console = dest.is_none();

        // access the tokens
        let contents = fs::read_to_string(&path)?;
        let src_tokens = VhdlTokenizer::from_source_code(&contents).into_tokens_all();

        // perform a search on tokens
        let (start, end) = {
            // get the tokens
            let start_tokens = match &self.start {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let end_tokens = match &self.end {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            let comment_tokens = match &self.comment {
                Some(tokenizer) => tokenizer
                    .as_tokens_all()
                    .into_iter()
                    .filter(|t| t.as_type() != &VhdlToken::EOF)
                    .collect(),
                None => Vec::new(),
            };
            // search over the source code tokens
            let start = Self::find_location(&src_tokens, &start_tokens);
            // limit based on starting index
            let remaining_tokens = match &start {
                Some(pos) => src_tokens
                    .into_iter()
                    .skip_while(|p| p.locate() <= pos)
                    .collect(),
                None => {
                    if self.start.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'start' code chunk"
                        )))?;
                    }
                    src_tokens
                }
            };

            let end = Self::find_location(&remaining_tokens, &end_tokens);
            let remaining_tokens = match &end {
                Some(pos) => remaining_tokens
                    .into_iter()
                    .take_while(|p| p.locate() < pos)
                    .collect(),
                None => {
                    if self.end.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'end' code chunk"
                        )))?;
                    }
                    remaining_tokens
                }
            };

            // find the comment
            let first_token = Self::find_location(&remaining_tokens, &comment_tokens);
            // grab all continuous '--' tokens immediately above/before this location
            let comment = match &first_token {
                Some(pos) => {
                    let mut line = pos.line();
                    // println!("{:?}", pos);
                    match remaining_tokens
                        .into_iter()
                        .rev()
                        .skip_while(|p| p.locate() >= pos)
                        .take_while(|p| {
                            // only take immediate comments grouped together
                            line -= 1;
                            p.as_type().as_comment().is_some() && p.locate().line() == line
                        })
                        .last()
                    {
                        Some(token) => Some(token.locate().clone()),
                        None => {
                            return Err(AnyError(format!(
                                "Zero comments associated with code chunk"
                            )))?
                        }
                    }
                }
                None => {
                    if self.comment.is_some() == true {
                        return Err(AnyError(format!(
                            "Failed to find code segment matching 'doc' code chunk"
                        )))?;
                    }
                    None
                }
            };

            let end = match &comment {
                Some(_) => first_token,
                None => end,
            };

            let start = match comment {
                Some(c) => Some(c),
                None => start,
            };

            (start, end)
        };

        let segment: String = {
            let iter = contents.split_terminator('\n');

            let iter = match &start {
                Some(p) => iter.skip(p.line() - 1),
                None => iter.skip(0),
            };
            let iter = match &end {
                Some(p) => {
                    iter.take(p.line() - start.as_ref().unwrap_or(&Position::new()).line() + 1)
                }
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
                true => {
                    segment
                }
                // overwrite contents and display the file path
                false => {
                    let cut_code = (start.is_some() || end.is_some()) && self.location == false;

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
                        }
                        false => std::fs::OpenOptions::new().read(true).open(&path)?,
                    };

                    // set to read-only
                    let mut perms = file.metadata()?.permissions();
                    perms.set_readonly(true);
                    file.set_permissions(perms)?;

                    // tack on the location to the file name
                    let path = {
                        if self.location == true {
                            // update the location to point to
                            let loc = match start {
                                Some(s) => s,
                                None => match end {
                                    Some(e) => e,
                                    None => loc,
                                },
                            };
                            // append the location to the filepath
                            PathBuf::from({
                                let mut p = path.as_os_str().to_os_string();
                                p.push(&loc.to_string());
                                p
                            })
                        } else {
                            path
                        }
                    };

                    path.display().to_string()
                }
            }
        );
        Ok(())
    }

    fn check_tokens_eq(source: &VhdlToken, sub: &VhdlToken) -> bool {
        match sub {
            // skip EOF token
            &VhdlToken::EOF => true,
            // only match on the fact that they are comments
            VhdlToken::Comment(_) => source.as_comment().is_some(),
            _ => source == sub,
        }
    }

    fn find_location(
        src_tokens: &Vec<Token<VhdlToken>>,
        find_tokens: &Vec<&Token<VhdlToken>>,
    ) -> Option<Position> {
        let mut tracking: bool;
        let mut src_tokens_iter = src_tokens.iter();
        while let Some(t) = src_tokens_iter.next() {
            // begin to see if we start tracking
            // println!("{:?} {:?}", find_tokens.first().unwrap().as_type(), t.as_type());

            if find_tokens.len() > 0
                && Self::check_tokens_eq(t.as_type(), find_tokens.first().unwrap().as_type())
                    == true
            {
                // println!("{}", "HERE");
                let mut find_tokens_iter = find_tokens.iter().skip(1);
                tracking = true;
                while let Some(find_t) = find_tokens_iter.next() {
                    // skip the EOF token
                    if find_t.as_type() == &VhdlToken::EOF {
                        continue;
                    }
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
        unit: &LangIdentifier,
        ip: &Ip,
        dest: Option<&PathBuf>,
        mode: &LangMode,
    ) -> Result<(PathBuf, Position), Fault> {
        // find the unit
        let units = Ip::collect_units(true, ip.get_root(), mode, true, ip.into_public_list())?;

        // get the file data for the primary design unit
        let (source, position) = match units.get_key_value(unit) {
            Some((_, unit)) => (
                unit.get_source_code_file(),
                unit.get_symbol().unwrap().get_position().clone(),
            ),
            None => {
                return Err(GetError::SuggestShow(
                    GetError::UnitNotFound(
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
