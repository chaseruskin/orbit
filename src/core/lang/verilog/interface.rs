//
//  Copyright (C) 2022-2024  Chase Ruskin
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

use super::super::sv::token::{
    identifier::Identifier, keyword::Keyword, operator::Operator, token::SystemVerilogToken,
};
use serde_derive::Serialize;

#[derive(Debug, PartialEq)]
pub struct Expr(Option<Vec<SystemVerilogToken>>);

impl serde::Serialize for Expr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Some(expr) => serializer.serialize_str(&tokens_to_string(&expr)),
            None => serializer.serialize_none(),
        }
    }
}

pub type PortList = Vec<Port>;
pub type ParamList = Vec<Port>;

fn tokens_to_string(tokens: &Vec<SystemVerilogToken>) -> String {
    let mut result = String::new();
    // determine which delimiters to not add trailing spaces to
    let is_spaced_token = |d: &Operator| match d {
        Operator::ParenL
        | Operator::ParenR
        | Operator::BrackL
        | Operator::BrackR
        | Operator::Dot
        | Operator::Pow
        | Operator::Minus
        | Operator::Plus
        | Operator::Mult
        | Operator::Colon
        | Operator::Div => false,
        _ => true,
    };

    // determine which delimiters to not add have whitespace preceed
    let no_preceeding_whitespace = |d: &Operator| match d {
        Operator::Pow | Operator::Comma | Operator::BrackL => true,
        _ => false,
    };

    let force_trailing_whitespace = |d: &Operator| match d {
        Operator::Gt | Operator::Gte | Operator::Lt | Operator::Lte => true,
        _ => false,
    };

    // iterate through the tokens
    let mut iter = tokens.iter().peekable();

    while let Some(t) = iter.next() {
        let mut force_space = false;
        // determine if to add trailing space after the token
        let trailing_space = match t {
            SystemVerilogToken::Operator(d) => {
                force_space = force_trailing_whitespace(d);
                force_space || is_spaced_token(d)
            }
            SystemVerilogToken::Number(_) => false,
            _ => {
                // make sure the next token is not a tight token (no-spaced)
                if let Some(m) = iter.peek() {
                    match m {
                        SystemVerilogToken::Operator(d) => is_spaced_token(d),
                        _ => true,
                    }
                } else {
                    true
                }
            }
        };

        // push the token to the string
        result.push_str(&t.to_string());
        // handle adding whitespace after the token
        if trailing_space == true && iter.peek().is_some() {
            if force_space == false {
                // check what the next token is to determine if whitespace should be added before it
                if let Some(d) = iter.peek().unwrap().as_delimiter() {
                    // skip whitespace addition
                    if no_preceeding_whitespace(d) == true {
                        continue;
                    }
                } else if let Some(_n) = iter.peek().unwrap().as_number() {
                    continue;
                }
            }
            result.push_str(" ");
        }
    }
    result
}

pub fn get_port_by_name_mut<'a>(
    port_list: &'a mut PortList,
    name: &Identifier,
) -> Option<&'a mut Port> {
    let port = port_list.iter_mut().find(|i| &i.name == name)?;
    Some(port)
}

/// Updates the port list by letting the existing port with its identifier inherit its defined
/// attributes. If the new port is not found, then it is not added to the list if `add_if_missing` is false.
pub fn update_port_list<'a>(
    port_list: &'a mut PortList,
    new_port: Port,
    add_if_missing: bool,
) -> () {
    let port = port_list.iter_mut().find(|i| &i.name == &new_port.name);
    match port {
        Some(p) => p.inherit(&new_port),
        None => {
            if add_if_missing == true {
                port_list.push(new_port)
            } else {
                ()
            }
        }
    }
}

pub fn display_connections(
    port_list: &Vec<Port>,
    is_params: bool,
    prefix: &str,
    suffix: &str,
) -> String {
    let mut result = String::new();

    if port_list.is_empty() == false {
        result.push(' ');
        if is_params == true {
            result.push('#');
        }
        result.push('(');
    }

    port_list.iter().enumerate().for_each(|(i, p)| {
        result.push_str("\n  ");
        result.push_str(&&&p.into_connection(prefix, suffix));
        if i != port_list.len() - 1 {
            result.push_str(",")
        };
    });

    if port_list.is_empty() == false {
        result.push('\n');
        result.push(')');
        if is_params == true {
            result.push(' ');
        }
    }

    result
}

pub fn display_interface(port_list: &Vec<Port>, is_params: bool) -> String {
    let mut result = String::new();
    if port_list.is_empty() == false {
        result.push(' ');
        if is_params == true {
            result.push('#');
        }
        result.push('(');
    }

    port_list.iter().enumerate().for_each(|(i, p)| {
        result.push_str("\n  ");
        result.push_str(&&&p.into_declaration(true, is_params, "", ""));
        if i != port_list.len() - 1 {
            result.push_str(",")
        };
    });

    if port_list.is_empty() == false {
        result.push('\n');
        result.push(')');
    }
    result
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Inout,
    Input,
    Output,
    Ref,
}

fn default_mode() -> Keyword {
    Keyword::Input
}

#[derive(Debug, PartialEq)]
pub struct DataType {
    net: Option<Keyword>,
    is_signed: bool,
    data: Option<SystemVerilogToken>,
    range: Expr,
}

impl DataType {
    pub fn new() -> Self {
        Self {
            net: None,
            is_signed: false,
            data: None,
            range: Expr(None),
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        Self {
            is_signed: false,
            net: Some(Keyword::Wire),
            data: None,
            range: Expr(None),
        }
    }
}

impl serde::Serialize for DataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut result = String::new();
        if let Some(dt) = &self.data {
            result.push_str(&dt.to_string());
        }
        if let Some(rg) = &self.range.0 {
            result.push_str(&tokens_to_string(rg));
        }
        serializer.serialize_str(&result)
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Port {
    #[serde(rename = "identifier")]
    name: Identifier,
    #[serde(rename = "mode", default = "default_mode")]
    mode: Option<Keyword>,
    #[serde(rename = "type", default)]
    data_type: DataType,
    #[serde(rename = "default")]
    value: Expr,
    #[serde(skip_serializing)]
    is_reg: bool,
}

impl Port {
    pub fn is_port_direction(kw: Option<&Keyword>) -> bool {
        let kw = if let Some(k) = kw { k } else { return false };
        match kw {
            Keyword::Input | Keyword::Inout | Keyword::Output | Keyword::Ref => true,
            _ => false,
        }
    }

    pub fn into_connection(&self, prefix: &str, suffix: &str) -> String {
        let mut result = String::new();

        result.push_str(&Operator::Dot.to_string());
        result.push_str(&self.name.to_string());
        result.push_str(&Operator::ParenL.to_string());
        result.push_str(prefix);
        result.push_str(&self.name.to_string());
        result.push_str(suffix);
        result.push_str(&Operator::ParenR.to_string());
        result
    }

    pub fn into_declaration(
        &self,
        use_mode: bool,
        is_param: bool,
        prefix: &str,
        suffix: &str,
    ) -> String {
        let mut result = String::new();

        if use_mode == true {
            // display the port direction
            match is_param {
                true => {
                    result.push_str(
                        &self
                            .mode
                            .as_ref()
                            .unwrap_or(&Keyword::Parameter)
                            .to_string(),
                    );
                }
                false => {
                    result.push_str(&self.mode.as_ref().unwrap_or(&Keyword::Input).to_string());
                }
            }
            result.push(' ');
        }

        // we previously omitted the mode
        if use_mode == false {
            match is_param {
                true => {
                    result.push_str(
                        &self
                            .mode
                            .as_ref()
                            .unwrap_or(&Keyword::Parameter)
                            .to_string(),
                    );
                    result.push(' ');
                }
                false => {
                    if self.data_type.data.is_none() {
                        result.push_str(&Keyword::Wire.to_string());
                        result.push(' ');
                    }
                }
            }
        } else {
            if let Some(n) = &self.data_type.net {
                result.push_str(&n.to_string());
                result.push(' ');
            }

            // display the reg keyword
            if self.is_reg == true {
                result.push_str(&Keyword::Reg.to_string());
                result.push(' ');
            }
        }

        // display the datatype
        if let Some(d) = &self.data_type.data {
            result.push_str(&d.to_string());
            result.push(' ');
        }

        // display if signed
        if self.data_type.is_signed == true {
            result.push_str(&Keyword::Signed.to_string());
            result.push(' ');
        }

        // display the range
        if let Some(r) = &self.data_type.range.0 {
            // remove the space the comes before the range
            if result.is_empty() == false {
                result.pop();
            }
            result.push_str(&tokens_to_string(r));
            result.push(' ');
        }

        // prepend any prefix
        result.push_str(&prefix);

        // display the identifier
        result.push_str(&self.name.to_string());

        // append any suffix
        result.push_str(&suffix);

        // display the default value
        if let Some(v) = &self.value.0 {
            result.push_str(&format!(" = {}", tokens_to_string(v)));
        }

        result
    }

    pub fn with(name: Identifier) -> Self {
        Self {
            name: name,
            mode: None,
            data_type: DataType::new(),
            is_reg: false,
            value: Expr(None),
        }
    }

    pub fn new() -> Self {
        Self {
            name: Identifier::new(),
            mode: None,
            data_type: DataType::new(),
            is_reg: false,
            value: Expr(None),
        }
    }

    pub fn inherit(&mut self, rhs: &Port) {
        if self.mode.is_none() {
            self.mode = rhs.mode.clone();
        }

        if self.data_type.net.is_none() {
            self.data_type.net = rhs.data_type.net.clone();
        }

        if self.data_type.data.is_none() {
            self.data_type.data = rhs.data_type.data.clone();
        }

        if self.is_reg == false {
            self.is_reg = rhs.is_reg;
        }

        if self.data_type.is_signed == false {
            self.data_type.is_signed = rhs.data_type.is_signed;
        }

        if self.data_type.range.0.is_none() {
            if let Some(r) = &rhs.data_type.range.0 {
                self.data_type.range = Expr(Some(r.clone()));
            }
        }

        if self.value.0.is_none() {
            if let Some(r) = &rhs.value.0 {
                self.value = Expr(Some(r.clone()));
            }
        }
    }

    pub fn set_default(&mut self, tkns: Vec<SystemVerilogToken>) {
        self.value = Expr(Some(tkns));
    }

    pub fn clear_default(&mut self) {
        self.value = Expr(None);
    }

    pub fn set_direction(&mut self, kw: Keyword) {
        self.mode = Some(kw);
    }

    pub fn set_net_type(&mut self, kw: Keyword) {
        self.data_type.net = Some(kw);
    }

    pub fn set_reg(&mut self) {
        self.is_reg = true;
    }

    pub fn set_signed(&mut self) {
        self.data_type.is_signed = true;
    }

    pub fn set_range(&mut self, tkns: Vec<SystemVerilogToken>) {
        self.data_type.range = Expr(Some(tkns));
    }

    pub fn set_data_type(&mut self, tkn: SystemVerilogToken) {
        self.data_type.data = Some(tkn);
    }

    pub fn as_user_defined_data_type(&self) -> Option<&Identifier> {
        match &self.data_type.data {
            Some(t) => match t.as_identifier() {
                Some(id) => Some(id),
                None => None,
            },
            None => None,
        }
    }
}
