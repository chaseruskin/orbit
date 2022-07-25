use super::token::Identifier;

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

use crate::core::lexer;
use crate::core::vhdl::token::{VHDLToken, Keyword, Delimiter};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub struct IdentifierList(Vec<Identifier>);

impl IdentifierList {
    fn from_tokens<I>(tokens: &mut Peekable<I>) -> Self 
    where I: Iterator<Item=lexer::Token<VHDLToken>> {
        let mut inner = Vec::new();
        // accept first identifier
        inner.push(tokens.next().unwrap().as_ref().as_identifier().unwrap().clone());
        while let Some(tkn) = tokens.peek() {
            // continue on commas
            if tkn.as_ref().check_delimiter(&Delimiter::Comma) == true {
                tokens.next();
            // collect more identifiers
            } else if tkn.as_ref().as_identifier().is_some() {
                inner.push(tokens.next().unwrap().as_ref().as_identifier().unwrap().clone());
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
    where I: Iterator<Item=lexer::Token<VHDLToken>> {
        let mut inner = Vec::new();
        while let Some(tkn) = tokens.peek() {
            // exit case: encounter 'bus' or ':=' delimiter
            if tkn.as_ref().check_keyword(&Keyword::Bus) || tkn.as_ref().check_delimiter(&Delimiter::VarAssign) {
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
    where I: Iterator<Item=lexer::Token<VHDLToken>> {
        // take remanining tokens
        Self(tokens.map(|f| f.take()).collect())
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
        self.0.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.0.len()
    }
}

impl std::fmt::Display for StaticExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":= {}", tokens_to_string(&self.0))
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

fn tokens_to_string(tokens: &Vec<VHDLToken>) -> String {
    let mut result = String::new();
    // determine which delimiters to not add trailing spaces to
    let is_spaced_token = |d: &Delimiter| {
        match d {
            Delimiter::ParenL | Delimiter::ParenR |
            Delimiter::Dash | Delimiter::Plus | Delimiter::Star | Delimiter::FwdSlash => false,
            _ => true,
        }
    };
    // iterate through the tokens
    let mut iter = tokens.iter().peekable();
    while let Some(t) = iter.next() {
        let trailing_space = match t {
            VHDLToken::Delimiter(d) => is_spaced_token(d),
            _ => {
                // make sure the next token is not a tight token (no-spaced)
                if let Some(m) = iter.peek() {
                    match m {
                        VHDLToken::Delimiter(d) => is_spaced_token(d),
                        _ => true
                    }
                } else {
                    true
                }
            }
        };
        result.push_str(&t.to_string());
        if trailing_space == true && iter.peek().is_some() {
            result.push_str(" ");
        }
    }
    result
}

impl InterfaceDeclaration {
    fn into_interface_string(&self, offset: usize) -> String {
        format!("{}{5:<width$}: {}{}{} {}", 
            self.identifier, 
            { if self.mode.is_none() && self.initial_keyword.is_some() && self.initial_keyword.as_ref().unwrap() == &Keyword::Signal { "in ".to_owned() } else if self.mode.is_some() { self.mode.as_ref().unwrap().to_string() + " " } else { "".to_owned() } }, 
            tokens_to_string(&self.datatype.0),
            { if self.bus_present { "bus" } else { "" } },
            { if self.expr.is_some() { self.expr.as_ref().unwrap().to_string() } else { "".to_string() }},
            " ",
            width=offset-self.identifier.len()+1,
        ).trim_end().to_string()
    }

    /// Creates a declaration string to be copied into architecture declaration parts.
    /// 
    /// Note: `offset` is used for padding after the identifier string and before ':'.
    fn into_declaration_string(&self, def_keyword: &Keyword, offset: usize) -> String {
        format!("{} {}{5:<width$}: {} {}{}",
            self.initial_keyword.as_ref().unwrap_or(def_keyword), 
            self.identifier,  
            tokens_to_string(&self.datatype.0),
            { if self.bus_present { "bus " } else { "" } },
            { if self.expr.is_some() { self.expr.as_ref().unwrap().to_string() } else { "".to_string() }},
            " ",
            width=offset-self.identifier.len()+1,
        ).trim_end().to_string()
    }

    /// Creates an instantiation line to be copied into an architecture region.
    fn into_instance_string(&self, offset: usize) -> String {
        format!("{}{:<width$}=> {}", self.identifier, " ", self.identifier, width=offset-self.identifier.len()+1)
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
        let longest = self.0
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
    where I: Iterator<Item=lexer::Token<VHDLToken>> {
        // check if 'signal' keyword is present
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
            return None
        }
        // check if a mode exists
        let token = tokens.peek()?;
        let mode = if let Some(kw) = token.as_type().as_keyword() {
            match kw {
                Keyword::In | Keyword::Out | Keyword::Buffer 
                | Keyword::Linkage | Keyword::Inout => {
                    true
                }
                _ => false,
            }
        } else {
            false
        };
        let mode = if mode { Some(tokens.next().unwrap().take().take_keyword().unwrap()) } else { None };
        // collect the datatype
        let subtype = SubtypeIndication::from_tokens(tokens);

        // check if bus keyword is present
        let token = tokens.peek();
        let bus_present = if let Some(tkn) = token {
            tkn.as_ref().check_keyword(&Keyword::Bus)
        } else {
            false
        };
        if bus_present == true { tokens.next(); }

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
    pub fn to_interface_part_string(&self) -> String {
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        let mut result = String::from("(\n");
        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_str(";\n")
            }
            result.push_str(&format!("    {}", port.into_interface_string(offset)));
        }
        result + "\n);"
    }

    pub fn to_declaration_part_string(&self, def_keyword: Keyword) -> String {
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        let mut result = String::new();
        for port in &self.0 {
            result.push_str(&port.into_declaration_string(&def_keyword, offset));
            result.push_str(&";\n");
        }
        result
    }

    pub fn to_instantiation_part(&self) -> String {
        // auto-align by first finding longest offset needed
        let offset = self.longest_identifier();
        let mut result = String::from("map (\n");
        for port in &self.0 {
            if port != self.0.first().unwrap() {
                result.push_str(",\n")
            }
            result.push_str(&format!("    {}", port.into_instance_string(offset)));
        }
        result + "\n)"
    }
}


#[cfg(test)]
mod test {
    // @todo
}