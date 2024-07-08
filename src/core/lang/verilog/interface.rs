use super::{
    symbols::Statement,
    token::{identifier::Identifier, keyword::Keyword, operator::Operator, token::VerilogToken},
};

type Tokens = Vec<VerilogToken>;

pub type PortList = Vec<Port>;
pub type ParamList = Vec<Port>;

fn tokens_to_string(tokens: &Vec<VerilogToken>) -> String {
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
        Operator::Pow | Operator::Comma => true,
        _ => false,
    };
    // iterate through the tokens
    let mut iter = tokens.iter().peekable();
    while let Some(t) = iter.next() {
        // determine if to add trailing space after the token
        let trailing_space = match t {
            VerilogToken::Operator(d) => is_spaced_token(d),
            VerilogToken::Number(n) => false,
            _ => {
                // make sure the next token is not a tight token (no-spaced)
                if let Some(m) = iter.peek() {
                    match m {
                        VerilogToken::Operator(d) => is_spaced_token(d),
                        _ => true,
                    }
                } else {
                    true
                }
            }
        };
        result.push_str(&t.to_string());
        if trailing_space == true && iter.peek().is_some() {
            if let Some(d) = iter.peek().unwrap().as_delimiter() {
                // skip whitespace addition
                if no_preceeding_whitespace(d) == true {
                    continue;
                }
            } else if let Some(_n) = iter.peek().unwrap().as_number() {
                continue;
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

pub fn display_param_list(param_list: &ParamList) -> String {
    let mut result = String::new();
    if param_list.is_empty() == false {
        result.push(' ');
        result.push('#');
        result.push('(');
    }

    param_list.iter().enumerate().for_each(|(i, p)| {
        result.push_str("\n  ");
        result.push_str(&p.display_as_param());
        if i != param_list.len() - 1 {
            result.push_str(",")
        };
    });

    if param_list.is_empty() == false {
        result.push('\n');
        result.push(')');
    }
    result
}

pub fn display_port_list(port_list: &PortList) -> String {
    let mut result = String::new();
    if port_list.is_empty() == false {
        result.push(' ');
        result.push('(');
    }

    port_list.iter().enumerate().for_each(|(i, p)| {
        result.push_str("\n  ");
        result.push_str(&&p.display_as_port());
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
}

#[derive(Debug, PartialEq)]
pub struct Port {
    direction: Option<Keyword>,
    net_type: Option<Keyword>,
    is_reg: bool,
    is_signed: bool,
    range: Option<Tokens>,
    name: Identifier,
    value: Option<Tokens>,
}

fn display_statement(stmt: &Statement) -> String {
    stmt.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&x.as_type().to_string());
        acc
    })
}

impl Port {
    pub fn display_as_param(&self) -> String {
        let mut result = String::new();
        result.push_str(
            &self
                .direction
                .as_ref()
                .unwrap_or(&Keyword::Parameter)
                .to_string(),
        );
        result.push(' ');
        result.push_str(&self.name.to_string());
        if let Some(v) = &self.value {
            result.push_str(&format!(" = {}", tokens_to_string(v)));
        }
        result
    }

    pub fn display_as_port(&self) -> String {
        let mut result = String::new();

        // display the port direction
        result.push_str(
            &self
                .direction
                .as_ref()
                .unwrap_or(&Keyword::Input)
                .to_string(),
        );
        result.push(' ');

        // display the net type
        if let Some(n) = &self.net_type {
            result.push_str(&n.to_string());
            result.push(' ');
        }

        // display the reg keyword
        if self.is_reg == true {
            result.push_str(&Keyword::Reg.to_string());
            result.push(' ');
        }

        // display if signed
        if self.is_signed == true {
            result.push_str(&Keyword::Signed.to_string());
            result.push(' ');
        }

        // display the range
        if let Some(r) = &self.range {
            result.push_str(&tokens_to_string(r));
            result.push(' ');
        }

        // display the identifier
        result.push_str(&self.name.to_string());
        result
    }

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
                self.range = Some(r.clone());
            }
        }

        if self.value.is_none() {
            if let Some(r) = &rhs.value {
                self.value = Some(r.clone());
            }
        }
    }

    pub fn set_default(&mut self, tkns: Tokens) {
        self.value = Some(tkns);
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

    pub fn set_range(&mut self, tkns: Tokens) {
        self.range = Some(tkns);
    }
}
