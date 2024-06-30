use std::{collections::HashSet, fmt::Display};

use super::LangIdentifier;
use std::hash::Hash;
type VhdlIdentifier = super::vhdl::token::Identifier;
type VerilogIdentifier = super::verilog::token::identifier::Identifier;

pub type RefSet = HashSet<CompoundIdentifier>;

/// A `CompoundIdentifier` is a pattern in the code that catches `<library>.<primary-unit>`. We
/// assume the pattern can be found anywhere.
///
/// A special case is just a simple name (1 identifier) when referencing a component name.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct CompoundIdentifier {
    prefix: Option<LangIdentifier>,
    suffix: LangIdentifier,
}

impl Display for CompoundIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.prefix {
            Some(p) => write!(f, "{}.{}", p, self.suffix),
            None => write!(f, "{}", self.suffix),
        }
    }
}

impl CompoundIdentifier {
    pub fn new_vhdl(prefix: VhdlIdentifier, suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Vhdl(prefix)),
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn new(prefix: LangIdentifier, suffix: LangIdentifier) -> Self {
        Self {
            prefix: Some(prefix),
            suffix: suffix,
        }
    }

    pub fn new_verilog(prefix: VerilogIdentifier, suffix: VerilogIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Verilog(prefix)),
            suffix: LangIdentifier::Verilog(suffix),
        }
    }

    pub fn new_minimal_vhdl(suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: None,
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn entity(prefix: VhdlIdentifier, suffix: VhdlIdentifier) -> Self {
        Self {
            prefix: Some(LangIdentifier::Vhdl(prefix)),
            suffix: LangIdentifier::Vhdl(suffix),
        }
    }

    pub fn get_suffix(&self) -> &LangIdentifier {
        &self.suffix
    }

    pub fn get_prefix(&self) -> Option<&LangIdentifier> {
        self.prefix.as_ref()
    }

    /// Checks if the identifiers `prefix` and `suffix` align with the those of
    /// `self`. Ignores checking the `prefix` if self does not have a prefix.
    pub fn is_match(&self, prefix: &LangIdentifier, suffix: &LangIdentifier) -> bool {
        let first_match = match &self.prefix {
            Some(p) => p == prefix,
            None => true,
        };
        first_match && &self.suffix == suffix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lang::verilog::token::identifier::Identifier::Basic;
    use crate::core::lang::vhdl::token::identifier::Identifier::Basic as VBasic;
    use crate::core::lang::LangIdentifier::Verilog;
    use crate::core::lang::LangIdentifier::Vhdl;

    #[test]
    fn ut_equal() {
        let vhdl = CompoundIdentifier {
            prefix: None,
            suffix: Vhdl(VBasic("b".to_string())),
        };
        let verilog = CompoundIdentifier {
            prefix: None,
            suffix: Verilog(Basic("b".to_string())),
        };
        assert_eq!(vhdl, verilog);
    }
}
