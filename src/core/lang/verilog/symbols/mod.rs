use std::iter::Peekable;

use super::super::sv::token::operator::Operator;
use super::error::VerilogError;
use super::interface::{self, ParamList};
use super::token::identifier::Identifier;
use super::token::tokenizer::VerilogTokenizer;
use crate::core::lang::lexer::{Position, Token};
use crate::core::lang::parser::{Parse, Symbol};
use crate::core::lang::reference::{CompoundIdentifier, RefSet};
use crate::core::lang::sv::symbols::SystemVerilogSymbol;
use crate::core::lang::sv::token::keyword::Keyword;
use crate::core::lang::sv::token::token::SystemVerilogToken;
use crate::core::lang::verilog::interface::{Port, PortList};
use crate::core::lang::verilog::token::token::VerilogToken;
use std::str::FromStr;

pub mod config;
pub mod module;

use config::Config;
use module::Module;

pub type Statement = Vec<Token<SystemVerilogToken>>;

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

/// Design elements of the Verilog Language.
#[derive(Debug, PartialEq)]
pub enum VerilogSymbol {
    Module(Module),
    Config(Config),
}

impl VerilogSymbol {
    pub fn as_name(&self) -> Option<&Identifier> {
        match &self {
            Self::Module(m) => Some(m.get_name()),
            Self::Config(c) => Some(c.get_name()),
        }
    }

    pub fn get_position(&self) -> &Position {
        match self {
            Self::Module(m) => m.get_position(),
            Self::Config(c) => c.get_position(),
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match &self {
            Self::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_config(&self) -> Option<&Config> {
        match &self {
            Self::Config(c) => Some(c),
            _ => None,
        }
    }

    pub fn get_refs(&self) -> &RefSet {
        match &self {
            Self::Module(m) => m.get_refs(),
            Self::Config(c) => c.get_refs(),
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
        // up cast the tokens into SystemVerilog tokens (since SV is a superset)
        let tokens: Vec<Token<SystemVerilogToken>> = tokens
            .into_iter()
            .map(|m| {
                let (pos, tkn) = m.decouple();
                Token::new(SystemVerilogToken::from(tkn), pos)
            })
            .collect();

        let mut symbols = Vec::new();
        let mut tokens = tokens.into_iter().peekable();

        while let Some(t) = tokens.next() {
            // take attribute and ignore if okay
            if t.as_type().check_delimiter(&Operator::AttrL) {
                match VerilogSymbol::parse_attr(&mut tokens, t.into_position()) {
                    Ok(_) => (),
                    Err(e) => symbols.push(Err(e)),
                }
            }
            // create module symbol
            else if t.as_type().check_keyword(&Keyword::Module)
                || t.as_type().check_keyword(&Keyword::Macromodule)
            {
                symbols.push(
                    match VerilogSymbol::parse_module(&mut tokens, t.into_position()) {
                        Ok(module) => Ok(Symbol::new(module)),
                        Err(e) => Err(e),
                    },
                );
            // skip comments
            } else if t.as_type().check_keyword(&Keyword::Config) {
                symbols.push(
                    match VerilogSymbol::parse_config(&mut tokens, t.into_position()) {
                        Ok(config) => Ok(Symbol::new(config)),
                        Err(e) => Err(e),
                    },
                );
            // skip any potential illegal/unknown tokens at global scale
            } else if t.as_type().is_eof() == false {
                // println!("{:?}", t);
                // symbols.push(Err(VerilogError::Vague))
                continue;
            }
        }
        // println!("{:#?}", symbols);
        symbols
    }
}

impl VerilogSymbol {
    /// Parses an `Config` design element from the config's identifier to
    /// the END closing statement.
    fn parse_config<I>(
        tokens: &mut Peekable<I>,
        pos: Position,
    ) -> Result<VerilogSymbol, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(VerilogSymbol::Config(Config::from_tokens(tokens, pos)?))
    }

    fn parse_assignment<I>(
        tokens: &mut Peekable<I>,
        take_separator: bool,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
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
        beg_t: Token<SystemVerilogToken>,
        end_op: Operator,
    ) -> Result<Statement, VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
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
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        Ok(VerilogSymbol::Module(Module::from_tokens(tokens, pos)?))
    }

    pub fn parse_module_declaration<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(ParamList, PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
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
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        let mut params = ParamList::new();
        let mut ports = PortList::new();
        let mut refs = RefSet::new();
        let mut deps = RefSet::new();

        let mut current_stmt = Statement::new();
        let mut prev_stmt_used_begin = false;

        while let Some(t) = tokens.next() {
            // expecting `endmodule`
            if t.as_ref().is_eof() == true {
                return Err(VerilogError::ExpectingKeyword(Keyword::Endmodule));
            // exit from the module architecture
            } else if t.as_ref().check_keyword(&Keyword::Endmodule) == true {
                break;
            // take a block's optional name
            } else if prev_stmt_used_begin && t.as_ref().check_delimiter(&Operator::Colon) {
                current_stmt.push(t);
                current_stmt.push(tokens.next().unwrap());
                prev_stmt_used_begin = false;
                current_stmt = Self::handle_statement(
                    current_stmt,
                    &mut params,
                    &mut ports,
                    &mut refs,
                    &mut deps,
                )?;
            } else if Self::is_statement_separator(t.as_ref()) {
                // check if it is 'begin' and has a trailing ':'
                prev_stmt_used_begin = t.as_ref().check_keyword(&Keyword::Begin);
                // handle current statement
                current_stmt = Self::handle_statement(
                    current_stmt,
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
                if opening_p.as_ref().check_delimiter(&Operator::ParenL) {
                    // take the parentheses
                    current_stmt.extend(Self::parse_until_operator(
                        tokens,
                        opening_p,
                        Operator::ParenR,
                    )?);
                }
                // handle the statement
                current_stmt = Self::handle_statement(
                    current_stmt,
                    &mut params,
                    &mut ports,
                    &mut refs,
                    &mut deps,
                )?;
                prev_stmt_used_begin = false;
            } else {
                current_stmt.push(t);
            }
        }
        Ok((params, ports, refs, deps))
    }

    fn handle_statement(
        stmt: Statement,
        mut params: &mut ParamList,
        mut ports: &mut PortList,
        refs: &mut RefSet,
        deps: &mut RefSet,
    ) -> Result<Statement, VerilogError> {
        // println!("{:?}", statement_to_string(&stmt));
        if stmt.is_empty() == true {
            return Ok(Statement::new());
        }

        // update references that may appear in the statement
        if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
            refs.extend(s_refs);
        }

        if let Some(dep) = Self::as_module_instance(&stmt) {
            // println!("detected dependency! {}", dep);
            deps.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
            refs.insert(CompoundIdentifier::new_minimal_verilog(dep.clone()));
        }
        // try as a port
        if let Some(def_ports) = Self::as_port_definition(&stmt) {
            def_ports
                .into_iter()
                .for_each(|p| interface::update_port_list(&mut ports, p, true));
        }
        if let Some(def_params) = Self::as_param_definition(&stmt) {
            def_params
                .into_iter()
                .for_each(|p| interface::update_port_list(&mut params, p, true));
        }
        if stmt
            .first()
            .unwrap()
            .as_type()
            .check_keyword(&Keyword::Import)
        {
            let mut tokens = stmt.into_iter().skip(1).peekable();
            let i_refs = SystemVerilogSymbol::parse_import_statement(&mut tokens)?;
            refs.extend(i_refs);
        }

        // reset the statement
        Ok(Statement::new())
    }

    fn as_port_definition(stmt: &Statement) -> Option<PortList> {
        let mut ports = PortList::new();
        let mut current_port_config = Port::new();

        let mut counter = 0;
        let mut state = 0;
        let mut sub_stmt = Statement::new();
        let mut stmt_iter = stmt.iter().enumerate();
        while let Some((i, t)) = stmt_iter.next() {
            match state {
                0 => {
                    // we are dealing with a param list
                    if let Some(name) = t.as_ref().as_identifier() {
                        if i == 0 {
                            state = -1;
                        }
                        // collect all names until something else is hit
                        ports.push(Port::with(name.clone()));
                        ports.last_mut().unwrap().inherit(&current_port_config);
                    } else if t.as_ref().check_delimiter(&Operator::Comma) {
                        // proceed
                        // clear the default value (if exists)
                        current_port_config.clear_default();
                        continue;
                    // we are dealing with parameter declarations
                    } else if t.as_ref().check_keyword(&Keyword::Input)
                        || t.as_ref().check_keyword(&Keyword::Output)
                        || t.as_ref().check_keyword(&Keyword::Inout)
                        || t.as_ref().check_keyword(&Keyword::Ref)
                    {
                        current_port_config = Port::new();
                        current_port_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
                    // collect a range
                    } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                        sub_stmt.push(t.clone());
                        state = 1;
                    // collect a default value
                    } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                        state = 2;
                    } else if t.as_ref().check_keyword(&Keyword::Reg) {
                        current_port_config.set_reg();
                    } else if t.as_ref().check_keyword(&Keyword::Signed) {
                        current_port_config.set_signed();
                    } else if Self::is_valid_net_type(t.as_ref().as_keyword()) {
                        current_port_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
                    } else if Self::is_valid_data_type(t.as_ref()) {
                        current_port_config.set_data_type(t.as_ref().clone());
                    } else {
                        state = -1;
                    }
                }
                // collect a range
                1 => {
                    sub_stmt.push(t.clone());
                    if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::BrackR) {
                        if counter == 0 {
                            current_port_config.set_range(into_tokens(sub_stmt.clone()));
                            sub_stmt.clear();
                            state = 0;
                        } else {
                            counter -= 1;
                        }
                    }
                }
                // collect an assignment
                2 => {
                    if t.as_ref().check_delimiter(&Operator::Comma) {
                        // set the default for the last known port!
                        ports
                            .last_mut()
                            .unwrap()
                            .set_default(into_tokens(sub_stmt.clone()));
                        sub_stmt.clear();
                        state = 0;
                    // parse nested parentheses
                    } else {
                        sub_stmt.push(t.clone());
                    }
                }
                _ => break,
            }
        }
        // fill the final default value if broke out of loop during that state (no more tokens)
        if sub_stmt.is_empty() == false && state == 2 {
            ports
                .last_mut()
                .unwrap()
                .set_default(into_tokens(sub_stmt.clone()));
        }
        match state >= 0 && counter == 0 {
            true => Some(ports),
            false => None,
        }
    }

    fn is_valid_net_type(op: Option<&Keyword>) -> bool {
        let op = match op {
            Some(op) => op,
            None => return false,
        };
        match op {
            // supply0 | supply1 | tri | triand | trior | tri0 | tri1 | uwire | wire | wand | wor
            Keyword::Wire
            | Keyword::Reg
            | Keyword::Supply0
            | Keyword::Supply1
            | Keyword::Tri
            | Keyword::Triand
            | Keyword::Trior
            | Keyword::Tri0
            | Keyword::Tri1
            | Keyword::Uwire
            | Keyword::Wand
            | Keyword::Wor => true,
            _ => false,
        }
    }

    fn is_valid_data_type(tkn: &SystemVerilogToken) -> bool {
        match tkn.as_identifier().is_some() {
            true => true,
            false => match tkn.as_keyword() {
                Some(kw) => match kw {
                    Keyword::Integer
                    | Keyword::Real
                    | Keyword::Time
                    | Keyword::Realtime
                    | Keyword::Logic
                    | Keyword::Bit
                    | Keyword::Byte
                    | Keyword::Shortint
                    | Keyword::Int
                    | Keyword::Longint
                    | Keyword::Shortreal => true,
                    _ => false,
                },
                None => false,
            },
        }
    }

    fn as_param_definition(stmt: &Statement) -> Option<ParamList> {
        let mut params = PortList::new();
        let mut current_param_config = Port::new();

        let mut counter = 0;
        let mut state = 0;
        let mut sub_stmt = Statement::new();
        let mut stmt_iter = stmt.iter();
        while let Some(t) = stmt_iter.next() {
            match state {
                0 => {
                    // we are dealing with a param list
                    if let Some(name) = t.as_ref().as_identifier() {
                        // collect all names until something else is hit
                        params.push(Port::with(name.clone()));
                        params.last_mut().unwrap().inherit(&current_param_config);
                    } else if t.as_ref().check_delimiter(&Operator::Comma) {
                        // proceed
                        // clear the default value (if exists)
                        current_param_config.clear_default();
                        continue;
                    // we are dealing with parameter declarations
                    } else if t.as_ref().check_keyword(&Keyword::Parameter) {
                        current_param_config = Port::new();
                        current_param_config
                            .set_direction(t.as_ref().as_keyword().unwrap().clone());
                    // collect a range
                    } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                        sub_stmt.push(t.clone());
                        state = 1;
                    // collect a default value
                    } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                        state = 2;
                    } else if t.as_ref().check_keyword(&Keyword::Reg) {
                        current_param_config.set_reg();
                    } else if t.as_ref().check_keyword(&Keyword::Signed) {
                        current_param_config.set_signed();
                    // this is the datatype...? for the parameter
                    } else if t.as_ref().as_keyword().is_some() {
                        current_param_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
                    } else {
                        state = -1;
                    }
                }
                // collect a range
                1 => {
                    sub_stmt.push(t.clone());
                    if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::BrackR) {
                        if counter == 0 {
                            current_param_config.set_range(into_tokens(sub_stmt.clone()));
                            sub_stmt.clear();
                            state = 0;
                        } else {
                            counter -= 1;
                        }
                    }
                }
                // collect an assignment
                2 => {
                    if t.as_ref().check_delimiter(&Operator::Comma) {
                        // set the default for the last known port!
                        params
                            .last_mut()
                            .unwrap()
                            .set_default(into_tokens(sub_stmt.clone()));
                        sub_stmt.clear();
                        state = 0;
                    // parse nested parentheses
                    } else {
                        sub_stmt.push(t.clone());
                    }
                }
                _ => break,
            }
        }
        // fill the final default value if broke out of loop during that state (no more tokens)
        if sub_stmt.is_empty() == false && state == 2 {
            params
                .last_mut()
                .unwrap()
                .set_default(into_tokens(sub_stmt.clone()));
        }
        match state >= 0 && counter == 0 {
            true => Some(params),
            false => None,
        }
    }

    /// Returns the name of the module that is being instantiated in this statement, if
    /// one exists.
    fn as_module_instance(stmt: &Statement) -> Option<&Identifier> {
        let mod_name = stmt.first()?.as_ref().as_identifier()?;
        // are there parameters defined
        let mut stmt_iter = stmt.iter().skip(1);

        let mut state = 0;
        let mut counter = 0;
        while let Some(t) = stmt_iter.next() {
            // println!("{}", t.as_ref().to_string());
            // take the parameters
            match state {
                // take either name or parameters
                0 => {
                    if t.as_ref().check_delimiter(&Operator::Pound) {
                        state = 1;
                    } else if t.as_ref().as_identifier().is_some() {
                        state = 1;
                    } else if t.as_ref().check_delimiter(&Operator::Comma) {
                        state = 0;
                    } else {
                        state = -1;
                    }
                }
                // enter parameters or ports listings
                1 => {
                    // take port/parameter list
                    if t.as_ref().check_delimiter(&Operator::ParenL) {
                        counter = 0;
                        state = 3;
                    // take range specification
                    } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter = 0;
                        state = 4;
                    } else {
                        state = -1;
                    }
                }
                // take until closing parenthesis
                3 => {
                    if t.as_ref().check_delimiter(&Operator::ParenL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::ParenR) {
                        if counter == 0 {
                            state = 0;
                        } else {
                            counter -= 1;
                        }
                    }
                }
                // take until closing bracket
                4 => {
                    if t.as_ref().check_delimiter(&Operator::BrackL) {
                        counter += 1;
                    } else if t.as_ref().check_delimiter(&Operator::BrackR) {
                        if counter == 0 {
                            // go to state 1 next
                            state = 1;
                        } else {
                            counter -= 1;
                        }
                    }
                }
                _ => break,
            }
        }
        match state >= 0 && counter == 0 {
            true => Some(mod_name),
            false => None,
        }
    }

    /// Checks if this is special token to take a statement using parentheses
    fn is_start_to_parentheses_statement(t: &SystemVerilogToken) -> bool {
        match t {
            SystemVerilogToken::Keyword(k) => match k {
                Keyword::If
                | Keyword::For
                | Keyword::Casex
                | Keyword::Casez
                | Keyword::While
                | Keyword::Repeat
                | Keyword::Case => true,
                _ => false,
            },
            SystemVerilogToken::Operator(o) => match o {
                Operator::At => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn is_statement_separator(t: &SystemVerilogToken) -> bool {
        match t {
            SystemVerilogToken::Keyword(k) => match k {
                Keyword::Initial
                | Keyword::Begin
                | Keyword::End
                | Keyword::Else
                | Keyword::Endconfig
                | Keyword::Endfunction
                | Keyword::Endgenerate
                | Keyword::Endmodule
                | Keyword::Endprimitive
                | Keyword::Endspecify
                | Keyword::Endtable
                | Keyword::Endtask
                | Keyword::Endcase => true,
                _ => false,
            },
            SystemVerilogToken::Operator(o) => match o {
                Operator::Terminator | Operator::AttrR => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn parse_module_param_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(ParamList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // println!("{}", "PARSE PARAMS");

        let mut params = ParamList::new();
        let mut current_param_config = Port::new();

        let mut refs = RefSet::new();

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
                // update references that might have appeared in range
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                current_param_config.set_range(into_tokens(stmt));
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // set the default for the last known port!
                params.last_mut().unwrap().set_default(into_tokens(stmt));
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_param_config.set_reg();
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_param_config.set_signed();
            } else if Self::is_valid_net_type(t.as_ref().as_keyword()) {
                current_param_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if Self::is_valid_data_type(t.as_ref()) {
                current_param_config.set_data_type(t.as_ref().clone());
            }
        }
        // println!("{:?}", params);
        Ok((params, refs))
    }

    fn parse_module_port_list<I>(
        tokens: &mut Peekable<I>,
    ) -> Result<(PortList, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        // println!("{}", "PARSE PORTS");

        let mut ports = PortList::new();
        let mut current_port_config = Port::new();
        let mut refs = RefSet::new();

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
                || t.as_ref().check_keyword(&Keyword::Ref)
            {
                current_port_config = Port::new();
                current_port_config.set_direction(t.as_ref().as_keyword().unwrap().clone());
            // collect a range
            } else if t.as_ref().check_delimiter(&Operator::BrackL) {
                let stmt = Self::parse_until_operator(tokens, t, Operator::BrackR)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                current_port_config.set_range(into_tokens(stmt));
            // collect a default value
            } else if t.as_ref().check_delimiter(&Operator::BlockAssign) {
                let stmt = Self::parse_assignment(tokens, false)?;
                // update references that may appear in the assignment
                if let Some(s_refs) = SystemVerilogSymbol::extract_refs_from_statement(&stmt) {
                    refs.extend(s_refs);
                }
                // set the default for the last known port!
                ports.last_mut().unwrap().set_default(into_tokens(stmt));
            } else if t.as_ref().check_keyword(&Keyword::Reg) {
                current_port_config.set_reg();
            } else if t.as_ref().check_keyword(&Keyword::Signed) {
                current_port_config.set_signed();
            } else if Self::is_valid_net_type(t.as_ref().as_keyword()) {
                current_port_config.set_net_type(t.as_ref().as_keyword().unwrap().clone());
            } else if Self::is_valid_data_type(t.as_ref()) {
                current_port_config.set_data_type(t.as_ref().clone());
            }
        }

        Ok((ports, refs))
    }

    fn parse_port_connection<I>(
        _tokens: &mut Peekable<I>,
    ) -> Result<(Vec<Statement>, Vec<Statement>, RefSet), VerilogError>
    where
        I: Iterator<Item = Token<SystemVerilogToken>>,
    {
        todo!()
    }
}
