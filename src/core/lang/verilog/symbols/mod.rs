use std::collections::HashSet;
use std::iter::Peekable;

use super::error::VerilogError;
use super::token::identifier::Identifier;
use super::token::operator::Operator;
use super::token::tokenizer::VerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::RefSet;
use crate::core::lang::verilog::interface::{Port, PortList};
use crate::core::lang::verilog::token::keyword::Keyword;
use crate::core::lang::verilog::token::token::VerilogToken;
use std::str::FromStr;

pub mod module;

use module::Module;

pub type Statement = Vec<Token<VerilogToken>>;

#[derive(Debug, PartialEq)]
pub enum VerilogSymbol {
    // primary design units (verilog only has 1 haha)
    Module(Module),
    // other "design units" / things that can exist at the top level
}

impl VerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            _ => None,
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

impl Parse<VerilogToken> for VerilogParser {
    type SymbolType = VerilogSymbol;
    type SymbolError = VerilogError;

    fn parse(
        tokens: Vec<Token<VerilogToken>>,
    ) -> Vec<Result<Symbol<Self::SymbolType>, Self::SymbolError>> {
        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        let mut module_attr: Option<Statement> = None;

        while let Some(t) = tokens.next() {
            // take directives and ignore if okay
            if t.as_ref().is_directive() == true {
                continue;
            }
            // take attribute and ignore if okay
            else if t.as_ref().check_delimiter(&Operator::AttrL) {
                match VerilogSymbol::parse_attr(&mut tokens, t.into_position()) {
                    Ok(r) => module_attr = Some(r),
                    Err(e) => symbols.push(Err(e)),
                }
            }
            // create module symbol
            else if t.as_ref().check_keyword(&Keyword::Module)
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
                module_attr = None;
            // skip comments
            } else if t.as_type().as_comment().is_some() == true {
                continue;
            } else if t.as_type().is_eof() == false {
                println!("{:?}", t);
                // illegal tokens at global scope?
                symbols.push(Err(VerilogError::Vague))
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VerilogSymbol {
    // fn parse_directive<I>(
    //     tokens: &mut Peekable<I>,
    //     pos: Position,
    // ) -> Result<(), VerilogError>
    // where
    //     I: Iterator<Item = Token<VerilogToken>>,
    // {
    //     // take until a newline (this is not formally correct but will be OK for now)
    //     while let Some(t) = tokens.peek() {
    //         if t.locate().line() > pos.line() {
    //             break;
    //         } else {
    //             tokens.next();
    //         }
    //     }
    //     Ok(())
    // }

    fn parse_attr<I>(tokens: &mut Peekable<I>, pos: Position) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut stmt = vec![Token::new(VerilogToken::Operator(Operator::AttrL), pos)];
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.next() {
            if t.as_ref().check_delimiter(&Operator::AttrR) == true {
                stmt.push(t);
                break;
            } else if t.as_ref().is_eof() == true {
                // expecting closing attribute operator
                return Err(VerilogError::ExpectingOperator(Operator::AttrR));
            }
            stmt.push(t);
        }
        Ok(stmt)
    }

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

        // check if there are parameters
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ModDecIncomplete);
            // parse parameter list (optional)
            } else if t.as_ref().check_delimiter(&Operator::Pound) == true {
                let t_next = tokens.next().unwrap();
                if t_next.as_ref().check_delimiter(&Operator::ParenL) == true {
                    // parse parameter list
                    Self::parse_module_param_list(tokens)?;
                } else {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                }
            // parse port list (optional?)
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                Self::parse_module_port_list(tokens)?;
            // stop parsing the
            } else if t.as_ref().check_delimiter(&Operator::Terminator) == true {
                break;
            }
        }

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

        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                // expecting `endmodule`
                return Err(VerilogError::ExpectingKeyword(Keyword::Endmodule));
            // exit from the module architecture
            } else if t.as_ref().check_keyword(&Keyword::Endmodule) == true {
                break;
            }
        }
        Ok((params, ports, refs))
    }

    fn parse_module_param_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        println!("{}", "PARSE PARAMS");
        let mut counter = 1;
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the parameter checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                counter -= 1;
                if counter == 0 {
                    break;
                }
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            }
        }
        Ok((Vec::new(), Vec::new(), RefSet::new()))
    }

    fn parse_module_port_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        println!("{}", "PARSE PORTS");

        let mut port_map = PortList::new();

        let mut current_port_config = Port::new();

        let mut counter = 1;
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the parameter checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                counter -= 1;
                if counter == 0 {
                    break;
                }
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            // we are dealing with a port list
            } else if let Some(name) = t.as_ref().as_identifier() {
                // collect all names until something else is hit
                port_map.push(Port::with(name.clone()));
                port_map.last_mut().unwrap().inherit(&current_port_config);
            } else if t.as_ref().check_delimiter(&Operator::Comma) {
                // proceed
                continue;
            // we are dealing with port connections
            } else if t.as_ref().check_delimiter(&Operator::Dot) {
                todo!();
            // handle an attribute
            } else if t.as_ref().check_delimiter(&Operator::AttrL) {
                Self::parse_attr(tokens, t.into_position())?;
            // we are dealing with port declarations
            } else if t.as_ref().check_keyword(&Keyword::Input)
                || t.as_ref().check_keyword(&Keyword::Output)
                || t.as_ref().check_keyword(&Keyword::Inout)
            {
                current_port_config = Port::new();
                current_port_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
            }
        }
        println!("{:?}", port_map);
        Ok((Vec::new(), Vec::new(), RefSet::new()))
    }

    fn parse_port_connection<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        todo!()
    }
}
