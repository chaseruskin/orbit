use std::iter::Peekable;

use super::error::VerilogError;
use super::interface::ParamList;
use super::token::identifier::Identifier;
use super::token::operator::Operator;
use super::token::tokenizer::VerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::verilog::interface::{Port, PortList};
use crate::core::lang::verilog::token::keyword::Keyword;
use crate::core::lang::verilog::token::token::VerilogToken;
use std::str::FromStr;

pub mod module;

use module::Module;

pub type Statement = Vec<Token<VerilogToken>>;

fn statement_to_string(stmt: &Statement) -> String {
    stmt.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&x.as_type().to_string());
        acc.push(' ');
        acc
    })
}

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
        let symbols = VerilogParser::parse(
            VerilogTokenizer::from_str(&s)?
                .into_tokens()
                .into_iter()
                .filter(|s| s.as_type().is_comment() == false)
                .collect(),
        );
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
            // println!("{:?}", t);
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

    fn parse_assignment<I>(
        tokens: &mut Peekable<I>,
        take_separator: bool,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut stmt = Vec::new();
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.peek() {
            if t.as_ref().check_delimiter(&Operator::Terminator)
                || t.as_ref().check_delimiter(&Operator::Comma)
                || t.as_ref().check_delimiter(&Operator::ParenR)
            {
                // do not take the ';' operator or ',' operator unless told to
                if take_separator == true {
                    tokens.next();
                }
                break;
            } else if t.as_ref().is_eof() == true {
                tokens.next();
                // expecting closing attribute operator
                return Err(VerilogError::ExpectingOperator(Operator::Terminator));
            // parse nested parentheses
            } else if t.as_ref().check_delimiter(&Operator::ParenL) {
                let t = tokens.next().unwrap();
                stmt.extend(Self::parse_until_operator(tokens, t, Operator::ParenR)?);
            } else {
                let t = tokens.next().unwrap();
                stmt.push(t);
            }
        }
        Ok(stmt)
    }

    fn parse_until_operator<I>(
        tokens: &mut Peekable<I>,
        beg_t: Token<VerilogToken>,
        end_op: Operator,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut counter = 0;
        let mut stmt = vec![beg_t];
        let beg_op = stmt
            .first()
            .unwrap()
            .as_type()
            .as_delimiter()
            .unwrap()
            .clone();
        // keep taking tokens until the closing attribute
        while let Some(t) = tokens.next() {
            if t.as_ref().check_delimiter(&end_op) == true {
                stmt.push(t);
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&beg_op) {
                counter += 1;
                stmt.push(t);
            } else if t.as_ref().is_eof() == true {
                // expecting closing attribute operator
                if counter > 1 {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenR));
                } else if counter < 0 {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                } else {
                    return Err(VerilogError::ExpectingOperator(end_op));
                }
            } else {
                stmt.push(t);
            }
        }
        Ok(stmt)
    }

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
    ) -> Result<(ParamList, PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut param_list = ParamList::new();
        let mut port_list = PortList::new();
        let mut refs = RefSet::new();

        // check if there are parameters
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ModDecIncomplete);
            // parse parameter list (optional)
            } else if t.as_ref().check_delimiter(&Operator::Pound) == true {
                let t_next = tokens.next().unwrap();
                if t_next.as_ref().check_delimiter(&Operator::ParenL) == true {
                    // parse parameter list
                    let (params, param_refs) = Self::parse_module_param_list(tokens)?;
                    param_list.extend(params);
                    refs.extend(param_refs);
                } else {
                    return Err(VerilogError::ExpectingOperator(Operator::ParenL));
                }
            // parse port list (optional?)
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                let (ports, port_refs) = Self::parse_module_port_list(tokens)?;
                port_list.extend(ports);
                refs.extend(port_refs);
            // stop parsing the declaration
            } else if t.as_ref().check_delimiter(&Operator::Terminator) == true {
                break;
            }
        }

        Ok((param_list, port_list, refs))
    }

    pub fn parse_module_architecture<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(ParamList, PortList, RefSet, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        let mut params = ParamList::new();
        let mut ports = PortList::new();
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        let mut current_stmt = Statement::new();

        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                // expecting `endmodule`
                return Err(VerilogError::ExpectingKeyword(Keyword::Endmodule));
            // exit from the module architecture
            } else if t.as_ref().check_keyword(&Keyword::Endmodule) == true {
                break;
            } else if Self::is_statement_separator(t.as_ref()) {
                // handle current statement
                Self::handle_statement(
                    &mut current_stmt,
                    &mut params,
                    &mut ports,
                    &mut refs,
                    &mut deps,
                )?;
            } else if Self::is_start_to_parentheses_statement(t.as_ref()) {
                // push token to the statement
                current_stmt.push(t);
                // expecting '('
                let opening_p = tokens.next().unwrap();
                // take the parentheses
                current_stmt.extend(Self::parse_until_operator(
                    tokens,
                    opening_p,
                    Operator::ParenR,
                )?);
                // handle the statement
                Self::handle_statement(
                    &mut current_stmt,
                    &mut params,
                    &mut ports,
                    &mut refs,
                    &mut deps,
                )?;
            } else {
                current_stmt.push(t);
            }
        }
        Ok((params, ports, refs, deps))
    }

    fn handle_statement(
        stmt: &mut Statement,
        params: &mut ParamList,
        ports: &mut PortList,
        refs: &mut RefSet,
        deps: &mut RefSet,
    ) -> Result<(), VerilogError> {
        println!("{:?}", statement_to_string(&stmt));
        if stmt.is_empty() == true {
            return Ok(());
        }
        if let Some(dep) = stmt.first().unwrap().as_ref().as_identifier() {
            println!("detected dependency! {}", dep);
            deps.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
            refs.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
        }
        // reset the statement
        stmt.clear();
        Ok(())
    }

    /// Checks if this is special token to take a statement using parentheses
    fn is_start_to_parentheses_statement(t: &VerilogToken) -> bool {
        match t {
            VerilogToken::Keyword(k) => match k {
                Keyword::If | Keyword::For | Keyword::Case => true,
                _ => false,
            },
            VerilogToken::Operator(o) => match o {
                Operator::At => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn is_statement_separator(t: &VerilogToken) -> bool {
        match t {
            VerilogToken::Keyword(k) => match k {
                Keyword::Initial
                | Keyword::Begin
                | Keyword::End
                | Keyword::Else
                | Keyword::Endcase => true,
                _ => false,
            },
            VerilogToken::Operator(o) => match o {
                Operator::Terminator => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn parse_module_param_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(ParamList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        println!("{}", "PARSE PARAMS");

        let mut params = ParamList::new();
        let mut current_param_config = Port::new();

        let mut counter = 0;
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the param checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            // we are dealing with a param list
            } else if let Some(name) = t.as_ref().as_identifier() {
                // collect all names until something else is hit
                params.push(Port::with(name.clone()));
                params.last_mut().unwrap().inherit(&current_param_config);
            } else if t.as_ref().check_delimiter(&Operator::Comma) {
                // proceed
                // clear the default value (if exists)
                current_param_config.clear_default();
                continue;
            // handle an attribute
            } else if t.as_ref().check_delimiter(&Operator::AttrL) {
                Self::parse_attr(tokens, t.into_position())?;
            // we are dealing with parameter declarations
            } else if t.as_ref().check_keyword(&Keyword::Parameter) {
                current_param_config = Port::new();
                current_param_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
            // collect a range
            } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                let stmt = Self::parse_until_operator(tokens, t, Operator::BrackR)?;
                current_param_config.set_range(stmt);
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // set the default for the last known port!
                params.last_mut().unwrap().set_default(stmt);
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_param_config.set_reg();
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_param_config.set_signed();
            } else if t.as_ref().as_keyword().is_some() {
                current_param_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            }
        }
        println!("{:?}", params);
        Ok((params, RefSet::new()))
    }

    fn parse_module_port_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<VerilogToken>>,
    {
        println!("{}", "PARSE PORTS");

        let mut ports = PortList::new();

        let mut current_port_config = Port::new();

        let mut counter = 0;
        while let Some(t) = tokens.next() {
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingOperator(Operator::ParenR));
            // exit the port checking
            } else if t.as_ref().check_delimiter(&Operator::ParenR) == true {
                if counter == 0 {
                    break;
                }
                counter -= 1;
            } else if t.as_ref().check_delimiter(&Operator::ParenL) == true {
                counter += 1;
            // we are dealing with a port list
            } else if let Some(name) = t.as_ref().as_identifier() {
                // collect all names until something else is hit
                ports.push(Port::with(name.clone()));
                ports.last_mut().unwrap().inherit(&current_port_config);
            } else if t.as_ref().check_delimiter(&Operator::Comma) {
                // proceed
                // clear the default value (if exists
                current_port_config.clear_default();
                continue;
            // we are dealing with port connections
            } else if t.as_ref().check_delimiter(&Operator::Dot) {
                todo!("handle port connections with dot operator");
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
            // collect a range
            } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                let stmt = Self::parse_until_operator(tokens, t, Operator::BrackR)?;
                current_port_config.set_range(stmt);
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // set the default for the last known port!
                ports.last_mut().unwrap().set_default(stmt);
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_port_config.set_reg();
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_port_config.set_signed();
            } else if t.as_ref().as_keyword().is_some() {
                current_port_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            }
        }
        println!("{:?}", ports);
        Ok((ports, RefSet::new()))
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
