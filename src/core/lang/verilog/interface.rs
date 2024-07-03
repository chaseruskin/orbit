use crate::core::lang::lexer::Token;

use super::{
    symbols::Statement,
    token::{identifier::Identifier, keyword::Keyword},
};

pub type PortList = Vec<Port>;
pub type ParamList = Vec<Port>;

pub fn get_port_by_name_mut<'a>(
    port_list: &'a mut PortList,
    name: &Identifier,
) -> Option<&'a mut Port> {
    let port = port_list.iter_mut().find(|i| &i.name == name)?;
    Some(port)
}

/// Updates the port list by either letting the existing port with its identifier inherit its defined
/// attributes.
pub fn update_port_list<'a>(port_list: &'a mut PortList, new_port: Port) -> () {
    let port = port_list.iter_mut().find(|i| &i.name == &new_port.name);
    match port {
        Some(p) => p.inherit(&new_port),
        None => port_list.push(new_port),
    }
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Inout,
    Input,
    Output,
}

#[derive(Debug, PartialEq)]
pub struct Port {
    direction: Option<Keyword>,
    net_type: Option<Keyword>,
    is_reg: bool,
    is_signed: bool,
    range: Option<Statement>,
    name: Identifier,
    value: Option<Statement>,
}

impl Port {
    pub fn with(name: Identifier) -> Self {
        Self {
            direction: None,
            net_type: None,
            is_reg: false,
            is_signed: false,
            range: None,
            name: name,
            value: None,
        }
    }

    pub fn new() -> Self {
        Self {
            direction: None,
            net_type: None,
            is_reg: false,
            is_signed: false,
            range: None,
            name: Identifier::new(),
            value: None,
        }
    }

    pub fn inherit(&mut self, rhs: &Port) {
        if self.direction.is_none() {
            self.direction = rhs.direction.clone();
        }

        if self.net_type.is_none() {
            self.net_type = rhs.net_type.clone();
        }

        if self.is_reg == false {
            self.is_reg = rhs.is_reg;
        }

        if self.is_signed == false {
            self.is_signed = rhs.is_signed;
        }

        if self.range.is_none() {
            if let Some(r) = &rhs.range {
                self.range = Some(
                    r.iter()
                        .map(|f| Token::new(f.as_type().clone(), f.locate().clone()))
                        .collect(),
                )
            }
        }

        if self.value.is_none() {
            if let Some(r) = &rhs.value {
                self.value = Some(
                    r.iter()
                        .map(|f| Token::new(f.as_type().clone(), f.locate().clone()))
                        .collect(),
                )
            }
        }
    }

    pub fn set_default(&mut self, stmt: Statement) {
        self.value = Some(stmt);
    }

    pub fn clear_default(&mut self) {
        self.value = None;
    }

    pub fn set_direction(&mut self, kw: Keyword) {
        self.direction = Some(kw);
    }

    pub fn set_net_type(&mut self, kw: Keyword) {
        self.net_type = Some(kw);
    }

    pub fn set_reg(&mut self) {
        self.is_reg = true;
    }

    pub fn set_signed(&mut self) {
        self.is_signed = true;
    }

    pub fn set_range(&mut self, stmt: Statement) {
        self.range = Some(stmt);
    }
}
