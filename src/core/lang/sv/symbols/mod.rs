use std::iter::Peekable;

use package::Package;

use super::error::SystemVerilogError;
use super::token::identifier::Identifier;
use super::token::operator::Operator;
use super::token::tokenizer::SystemVerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::RefSet;
use crate::core::lang::sv::token::keyword::Keyword;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::verilog::symbols::config::Config;
use std::str::FromStr;

use super::super::verilog::symbols::module::Module;

pub type Statement = Vec<Token<SystemVerilogToken>>;

pub mod package;

fn into_tokens(stmt: Statement) -> Vec<SystemVerilogToken> {
    stmt.into_iter().map(|t| t.take()).collect()
}

fn statement_to_string(stmt: &Statement) -> String {
    stmt.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&x.as_type().to_string());
        acc.push(' ');
        acc
    })
}

/// Design elements of the SystemVerilog Language.
#[derive(Debug, PartialEq)]
pub enum SystemVerilogSymbol {
    Module(Module),
    Config(Config),
    Package(Package),
}

impl SystemVerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            Self::Config(c) => Some(c.get_name()),
            Self::Package(p) => Some(p.get_name()),
        }
    }

    pub fn get_position(&self) -> &Position {
        match self {
            Self::Module(m) => m.get_position(),
            Self::Config(c) => c.get_position(),
            Self::Package(p) => p.get_position(),
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match &self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match &self {
            Self::Module(m) => m.get_refs(),
            Self::Config(c) => c.get_refs(),
            Self::Package(p) => p.get_refs(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SystemVerilogParser {
    symbols: Vec<Symbol<SystemVerilogSymbol>>,
}

impl SystemVerilogParser {
    /// Quietly ignores any errors and returns the list of symbols.
    pub fn read_lazy(s: &str) -> Self {
        let symbols =
            SystemVerilogParser::parse(SystemVerilogTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols
                .into_iter()
                .filter_map(|f| if f.is_ok() { Some(f.unwrap()) } else { None })
                .collect(),
        }
    }

    /// Reports an error if one is discovered in the list of symbols or in the tokenizing.
    pub fn read(s: &str) -> Result<Self, SystemVerilogError> {
        let symbols = SystemVerilogParser::parse(
            SystemVerilogTokenizer::from_str(&s)?
                .into_tokens()
                .into_iter()
                .filter(|s| s.as_type().is_comment() == false)
                .collect(),
        );
        let result: Result<Vec<Symbol<SystemVerilogSymbol>>, SystemVerilogError> =
            symbols.into_iter().collect();
        Ok(Self { symbols: result? })
    }

    pub fn into_symbols(self) -> Vec<SystemVerilogSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }
}

impl Parse<SystemVerilogToken> for SystemVerilogParser {
    type SymbolType = SystemVerilogSymbol;
    type SymbolError = SystemVerilogError;

    fn parse(
        tokens: Vec<Token<SystemVerilogToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        while let Some(t) = tokens.next() {
            // take attribute and ignore if okay
            if t.as_type().check_delimiter(&Operator::AttrL) {
                match SystemVerilogSymbol::parse_attr(&mut tokens, t.into_position()) {
                    Ok(_) => (),
                    Err(e) => symbols.push(Err(e)),
                }
            }
            // create module design element
            else if t.as_type().check_keyword(&Keyword::Module)
                || t.as_type().check_keyword(&Keyword::Macromodule)
            {
                symbols.push(
                    match SystemVerilogSymbol::parse_module(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // create package design element
            } else if t.as_type().check_keyword(&Keyword::Package) {
                symbols.push(
                    match SystemVerilogSymbol::parse_package(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // create config design element
            } else if t.as_type().check_keyword(&Keyword::Config) {
                symbols.push(
                    match SystemVerilogSymbol::parse_config(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
                        Err(e) => Err(e),
                    },
                )
            } else if t.as_type().is_eof() == false {
                // skip any potential illegal/unknown tokens at global scale
                // println!("{:?}", t);
                // illegal tokens at global scope?
                // symbols.push(Err(VerilogError::Vague))
                continue;
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl SystemVerilogSymbol {
    fn parse_module<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Module(Module::from_tokens(tokens, pos)?))
    }

    fn parse_package<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Package(Package::from_tokens(tokens, pos)?))
    }

    fn parse_config<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Self, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(Self::Config(Config::from_tokens(tokens, pos)?))
    }

    fn parse_attr<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<Statement, SystemVerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut stmt = vec![Token::new(
            SystemVerilogToken::Operator(Operator::AttrL),
            pos,
        )];
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.next() {
            if t.as_ref().check_delimiter(&Operator::AttrR) == true {
                stmt.push(t);
                break;
            } else if t.as_ref().is_eof() == true {
                // expecting closing attribute operator
                return Err(SystemVerilogError::ExpectingOperator(Operator::AttrR));
            }
            stmt.push(t);
        }
        Ok(stmt)
    }
}
