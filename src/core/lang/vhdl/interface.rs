use super::highlight::*;
use super::token::{Identifier, ToColor};
use colored::ColoredString;

pub fn library_statement(lib: &Identifier) -> String {
    format!(
        "{} {}{}\n",
        Keyword::Library.to_color(),
        color(&lib.to_string(), ENTITY_NAME),
        Delimiter::Terminator.to_color()
    )
}

#[derive(Debug, PartialEq)]
enum ColorTone {
    Color(ColoredString),
    Bland(String),
}

impl Display for ColorTone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Color(c) => write!(f, "{}", c),
            Self::Bland(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorVec(Vec<ColorTone>);

impl Display for ColorVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            write!(f, "{}", item)?
        }
        Ok(())
    }
}

impl ColorVec {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn push_str(&mut self, s: &str) -> () {
        self.0.push(ColorTone::Bland(String::from(s)));
    }

    fn push_color(&mut self, c: ColoredString) -> () {
        self.0.push(ColorTone::Color(c));
    }

    fn push(&mut self, ct: ColorTone) -> () {
        self.0.push(ct);
    }

    fn append(&mut self, mut cv: ColorVec) -> () {
        self.0.append(&mut cv.0);
    }

    fn push_whitespace(&mut self, count: usize) -> () {
        self.0
            .push(ColorTone::Bland(format!("{:<width$}", " ", width = count)));
    }

    fn swap(mut self, index: usize, hue: Rgb) -> Self {
        let item = self.0.get_mut(index).unwrap();
        *item = ColorTone::Color(color(&item.to_string(), hue));
        self
    }
}

#[derive(Debug)]
pub struct Architectures<'a>(&'a Vec<super::symbol::Architecture>);

impl<'a> Architectures<'a> {
    pub fn new(archs: &'a Vec<super::symbol::Architecture>) -> Self {
        Self(archs)
    }
}

impl<'a> std::fmt::Display for Architectures<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Architectures:\n")?;
        for arch in self.0 {
            write!(f, "    {}\n", arch.name())?
        }
        Ok(())
    }
}
// @note: interface_signal_declaration ::= [signal] identifier_list : [ mode ] subtype_indication [ bus ] [ := static_expression ]
// @note: identifier_list ::= identifier { , identifier }

use super::super::lexer;
use crate::core::lang::vhdl::token::{Delimiter, Keyword, VHDLToken};
use std::fmt::Display;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct IdentifierList(Vec<Identifier>);

impl IdentifierList {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = lexer::Token<VHDLToken>>,
    {
        let mut inner = Vec::new();
        // accept first identifier
        inner.push(
            tokens
                .next()
                .unwrap()
                .as_ref()
                .as_identifier()
                .unwrap()
                .clone(),
        );
        while let Some(tkn) = tokens.peek() {
            // continue on commas
            if tkn.as_ref().check_delimiter(&Delimiter::Comma) == true {
                tokens.next();
            // collect more identifiers
            } else if tkn.as_ref().as_identifier().is_some() {
                inner.push(
                    tokens
                        .next()
                        .unwrap()
                        .as_ref()
                        .as_identifier()
                        .unwrap()
                        .clone(),
                );
            // break on non-identifier or comma
            } else {
                break;
            }
        }
        Self(inner)
    }
}

#[derive(Debug, PartialEq)]
struct SubtypeIndication(Vec<VHDLToken>);

impl SubtypeIndication {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = lexer::Token<VHDLToken>>,
    {
        let mut inner = Vec::new();
        while let Some(tkn) = tokens.peek() {
            // exit case: encounter 'bus' or ':=' delimiter
            if tkn.as_ref().check_keyword(&Keyword::Bus)
                || tkn.as_ref().check_delimiter(&Delimiter::VarAssign)
            {
                break;
            } else {
                inner.push(tokens.next().unwrap().take());
            }
        }
        Self(inner)
    }
}

#[derive(Debug, PartialEq)]
struct StaticExpression(Vec<VHDLToken>);

impl StaticExpression {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self
    where
        I: Iterator<Item = lexer::Token<VHDLToken>>,
    {
        // take remanining tokens
        Self(tokens.map(|f| f.take()).collect())
    }

    fn to_color_vec(&self) -> ColorVec {
        let mut result = ColorVec::new();
        result.push_color(Delimiter::VarAssign.to_color());
        result.push_str(" ");
        result.append(tokens_to_string(&self.0));
        result
    }
}

#[derive(Debug, PartialEq)]
pub struct Generics(pub InterfaceDeclarations);

impl Generics {
    pub fn new() -> Self {
        Self(InterfaceDeclarations(Vec::new()))
    }
}

#[derive(Debug, PartialEq)]
pub struct Ports(pub InterfaceDeclarations);

impl Ports {
    pub fn new() -> Self {
        Self(InterfaceDeclarations(Vec::new()))
    }

    pub fn is_empty(&self) -> bool {
        self.0 .0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0 .0.len()
    }
}

impl std::fmt::Display for StaticExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", Delimiter::VarAssign, tokens_to_string(&self.0))
    }
}

#[derive(Debug, PartialEq)]
pub struct InterfaceDeclaration {
    initial_keyword: Option<Keyword>,
    identifier: Identifier,
    mode: Option<Keyword>,
    datatype: SubtypeIndication,
    bus_present: bool,
    expr: Option<StaticExpression>,
}

fn tokens_to_string(tokens: &Vec<VHDLToken>) -> ColorVec {
    let mut result = ColorVec::new();
    // determine which delimiters to not add trailing spaces to
    let is_spaced_token = |d: &Delimiter| match d {
        Delimiter::ParenL
        | Delimiter::ParenR
        | Delimiter::Dot
        | Delimiter::SingleQuote
        | Delimiter::Dash
        | Delimiter::Plus
        | Delimiter::Star
        | Delimiter::FwdSlash => false,
        _ => true,
    };
    // determine which delimiters to not add have whitespace preceed
    let no_preceeding_whitespace = |d: &Delimiter| match d {
        Delimiter::Comma => true,
        _ => false,
    };
    // iterate through the tokens
    let mut iter = tokens.iter().peekable();
    while let Some(t) = iter.next() {
        // determine if to add trailing space after the token
        let trailing_space = match t {
            VHDLToken::Delimiter(d) => is_spaced_token(d),
            _ => {
                // make sure the next token is not a tight token (no-spaced)
                if let Some(m) = iter.peek() {
                    match m {
                        VHDLToken::Delimiter(d) => is_spaced_token(d),
                        _ => true,
                    }
                } else {
                    true
                }
            }
        };
        result.push_color(t.to_color());
        if trailing_space == true && iter.peek().is_some() {
            if let Some(d) = iter.peek().unwrap().as_delimiter() {
                // skip whitespace addition
                if no_preceeding_whitespace(d) == true {
                    continue;
                }
            }
            result.push_str(" ");
        }
    }
    result
}

impl InterfaceDeclaration {
    fn into_interface_string(&self, offset: usize) -> ColorVec {
        let mut result = ColorVec::new();
        // identifier
        result.push_color(self.identifier.to_color());
        // whitespace
        result.push_whitespace(offset - self.identifier.len() + 1);
        // colon
        result.push_color(Delimiter::Colon.to_color());
        result.push_str(" ");
        // port direction
        if self.mode.is_none()
            && self.initial_keyword.is_some()
            && self.initial_keyword.as_ref().unwrap() == &Keyword::Signal
        {
            result.push_color(Keyword::In.to_color());
            result.push_str(" ");
        } else if self.mode.is_some() {
            result.push_color(self.mode.as_ref().unwrap().to_color());
            result.push_str(" ");
        }
        // data type
        result.append(tokens_to_string(&self.datatype.0).swap(0, DATA_TYPE));
        // optional bus keyword
        if self.bus_present == true {
            result.push_str(" ");
            result.push_color(Keyword::Bus.to_color())
        }
        // rhs initial assignment
        if self.expr.is_some() == true {
            result.push_str(" ");
            result.append(self.expr.as_ref().unwrap().to_color_vec())
        }
        return result;
    }

    /// Creates a declaration string to be copied into architecture declaration parts.
    ///
    /// Note: `offset` is used for padding after the identifier string and before ':'.
    fn into_declaration_string(&self, def_keyword: &Keyword, offset: usize) -> ColorVec {
        let mut result = ColorVec::new();

        result.push_color(
            self.initial_keyword
                .as_ref()
                .unwrap_or(def_keyword)
                .to_color(),
        );
        result.push_str(" ");
        result.push_color(color(&self.identifier.to_string(), SIGNAL_DEC_IDENTIFIER));
        result.push_whitespace(offset - self.identifier.len() + 1);
        result.push_color(Delimiter::Colon.to_color());
        result.push_str(" ");
        // data type
        result.append(tokens_to_string(&self.datatype.0).swap(0, DATA_TYPE));
        // optional bus keyword
        if self.bus_present == true {
            result.push_str(" ");
            result.push_color(Keyword::Bus.to_color())
        }
        // rhs initial assignment
        if self.expr.is_some() == true {
            result.push_str(" ");
            result.append(self.expr.as_ref().unwrap().to_color_vec())
        }
        result
    }

    /// Creates an instantiation line to be copied into an architecture region.
    fn into_instance_string(&self, offset: usize) -> ColorVec {
        let mut result = ColorVec::new();

        result.push_color(color(&self.identifier.to_string(), INSTANCE_LHS_IDENTIFIER));
        result.push_whitespace(offset - self.identifier.len() + 1);
        result.push_color(Delimiter::Arrow.to_color());
        result.push_str(" ");
        result.push_color(self.identifier.to_color());
        result
    }
}

#[derive(Debug, PartialEq)]
pub struct InterfaceDeclarations(Vec<InterfaceDeclaration>);

impl InterfaceDeclarations {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Determines the length of the longest identifier.
    pub fn longest_identifier(&self) -> usize {
        let longest = self
            .0
            .iter()
            .max_by(|x, y| x.identifier.len().cmp(&y.identifier.len()));
        match longest {
            Some(l) => l.identifier.len(),
            None => 0,
        }
    }

    /// Creates a set of `InterfaceDeclaration`s from VHDL Tokens.
    pub fn from_double_listed_tokens(tokens: Vec<Vec<lexer::Token<VHDLToken>>>) -> Self {
        let mut inner = Vec::new();
        for statement in tokens {
            match Self::from_tokens(&mut statement.into_iter().peekable()) {
                Some(mut interface) => inner.append(&mut interface.0),
                None => (),
            }
        }
        Self(inner)
    }

    /// Parses VHDL tokens into a series of `Interface` structs.
    pub fn from_tokens<I>(tokens: &mut Peekable<I>) -> Option<Self>
    where
        I: Iterator<Item = lexer::Token<VHDLToken>>,
    {
        // check if optional 'signal'/'constant'/'file'? keyword is present
        let token = tokens.peek()?;
        let initial_keyword = if token.as_ref().as_keyword().is_some() {
            Some(tokens.next().unwrap().take().take_keyword().unwrap())
        } else {
            None
        };
        // collect all identifiers for this type of signal
        let identifiers = IdentifierList::from_tokens(tokens);
        // skip past ':' delimiter
        if tokens.next()?.as_ref().check_delimiter(&Delimiter::Colon) == false {
            return None;
        }
        // check if a mode exists
        let token = tokens.peek()?;
        let mode = if let Some(kw) = token.as_type().as_keyword() {
            match kw {
                Keyword::In
                | Keyword::Out
                | Keyword::Buffer
                | Keyword::Linkage
                | Keyword::Inout => true,
                _ => false,
            }
        } else {
            false
        };
        let mode = if mode {
            Some(tokens.next().unwrap().take().take_keyword().unwrap())
        } else {
            None
        };
        // collect the datatype
        let subtype = SubtypeIndication::from_tokens(tokens);

        // check if bus keyword is present
        let token = tokens.peek();
        let bus_present = if let Some(tkn) = token {
            tkn.as_ref().check_keyword(&Keyword::Bus)
        } else {
            false
        };
        if bus_present == true {
            tokens.next();
        }

        // check if an expression exists
        let token = tokens.next();
        let expr = if let Some(tkn) = token {
            if tkn.as_ref().check_delimiter(&Delimiter::VarAssign) {
                Some(StaticExpression::from_tokens(tokens))
            } else {
                None
            }
        } else {
            None
        };

        // build signals
        let mut signals = Vec::new();
        for identifier in identifiers.0 {
            let signal = InterfaceDeclaration {
                initial_keyword: initial_keyword.clone(),
                identifier: identifier,
                mode: mode.clone(),
                datatype: SubtypeIndication(subtype.0.iter().map(|f| f.clone()).collect()),
                bus_present: bus_present,
                expr: match &expr {
                    Some(e) => Some(StaticExpression(e.0.iter().map(|f| f.clone()).collect())),
                    None => None,
                },
            };
            signals.push(signal);
        }
        Some(Self(signals))
    }

    /// Creates the body of the component list of interface connections.
    pub fn to_interface_part_string(&self) -> ColorVec {
        let mut result = ColorVec::new();
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        result.push_color(Delimiter::ParenL.to_color());
        result.push_str("\n");
        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_color(Delimiter::Terminator.to_color());
                result.push_str("\n");
            }
            result.push_whitespace(4);
            result.append(port.into_interface_string(offset));
        }
        result.push_str("\n");
        result.push_color(Delimiter::ParenR.to_color());
        result.push_color(Delimiter::Terminator.to_color());

        result
    }

    pub fn to_declaration_part_string(&self, def_keyword: Keyword) -> ColorVec {
        let mut result = ColorVec::new();
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        for port in &self.0 {
            result.append(port.into_declaration_string(&def_keyword, offset));
            result.push_color(Delimiter::Terminator.to_color());
            result.push_str("\n");
        }
        result
    }

    pub fn to_instantiation_part(&self) -> ColorVec {
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        let mut result = ColorVec::new();

        result.push_color(Keyword::Map.to_color());
        result.push_str(" ");
        result.push_color(Delimiter::ParenL.to_color());
        result.push_str("\n");

        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_color(Delimiter::Comma.to_color());
                result.push_str("\n");
            }
            result.push_whitespace(4);
            result.append(port.into_instance_string(offset));
        }
        result.push_str("\n");
        result.push_color(Delimiter::ParenR.to_color());
        result
    }
}

#[cfg(test)]
mod test {
    // @todo
}
