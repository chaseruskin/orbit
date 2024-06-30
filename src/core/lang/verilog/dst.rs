use std::collections::HashMap;

use crate::core::lang::{lexer::Token, LangIdentifier};

use super::token::token::VerilogToken;

pub fn dyn_symbol_transform(
    tkns: &[Token<VerilogToken>],
    lut: &HashMap<LangIdentifier, String>,
) -> String {
    todo!()
}
