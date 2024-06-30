use std::collections::HashSet;
use std::iter::Peekable;

use super::error::VerilogError;
use super::token::identifier::Identifier;
use super::token::tokenizer::VerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::RefSet;
use crate::core::lang::verilog::token::keyword::Keyword;
use crate::core::lang::verilog::token::token::VerilogToken;
use std::str::FromStr;

pub mod module;

use module::Module;

#[derive(Debug, PartialEq)]
pub enum VerilogSymbol {
    // primary design units (verilog only has 1 haha)
    Module(Module),
}

impl VerilogSymbol {
    pub fn as_name(&self) -> &Identifier {
        match &self {
            Self::Module(m) => m.get_name(),
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
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VerilogParser {
    symbols: Vec<Symbol<VerilogSymbol>>,
}

impl Parse<VerilogToken> for VerilogParser {
    type SymbolType = VerilogSymbol;
    type SymbolError = VerilogError;

    fn parse(
        tokens: Vec<Token<VerilogToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        //  let mut attrs = Vec::new();

        while let Some(t) = tokens.next() {
            // TODO: take directives
            // TODO: take attribute
            // create module symbol
            if t.as_ref().check_keyword(&Keyword::Module)
                || t.as_ref().check_keyword(&Keyword::Macromodule)
            {
                symbols.push(
                    match VerilogSymbol::parse_module(&mut tokens, t.into_position()) {
                        Ok(module) => {
                            // println!("info: detected {}", module);
                            // attrs = module.add_attributes(attrs);
                            Ok(Symbol::new(module))
                        }
                        Err(e) => Err(e),
                    },
                );
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VerilogParser {
    /// Quietly ignores any errors and returns the list of symbols.
    pub fn read_lazy(s: &str) -> Self {
        let symbols = VerilogParser::parse(VerilogTokenizer::from_source_code(&s).into_tokens());
        Self {
            symbols: symbols
                .into_iter()
                .filter_map(|f| if f.is_ok() { Some(f.unwrap()) } else { None })
                .collect(),
        }
    }

    /// Reports an error if one is discovered in the list of symbols or in the tokenizing.
    pub fn read(s: &str) -> Result<Self, VerilogError> {
        let symbols = VerilogParser::parse(VerilogTokenizer::from_str(&s)?.into_tokens());
        let result: Result<Vec<Symbol<VerilogSymbol>>, VerilogError> =
            symbols.into_iter().collect();
        Ok(Self { symbols: result? })
    }

    pub fn into_symbols(self) -> Vec<VerilogSymbol> {
        self.symbols.into_iter().map(|f| f.take()).collect()
    }
}

impl VerilogSymbol {
    /// Parses an `Entity` primary design unit from the entity's identifier to
    /// the END closing statement.
    fn parse_module<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VerilogSymbol, VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        Ok(VerilogSymbol::Module(Module::from_tokens(tokens, pos)?))
    }

    pub fn parse_module_declaration<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut params = Vec::new();
        let mut ports = Vec::new();
        let mut refs = HashSet::new();
        Ok((params, ports, refs))
    }

    pub fn parse_module_architecture<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut params = Vec::new();
        let mut ports = Vec::new();
        let mut refs = HashSet::new();
        Ok((params, ports, refs))
    }
}

#[derive(Debug, PartialEq)]
pub struct Statement(Vec<Token<VerilogToken>>);
