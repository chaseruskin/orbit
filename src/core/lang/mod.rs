pub mod verilog;
pub mod vhdl;

pub mod lexer;
pub mod parser;

pub mod node;


use serde_derive::Serialize;
use vhdl::primaryunit::PrimaryUnit;
use std::collections::HashMap;
use crate::util::anyerror::Fault;
use std::fmt::Display;
use toml_edit::InlineTable;
use std::str::FromStr;

type VhdlIdentifier = vhdl::token::Identifier;
use serde_derive::Deserialize;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum LangMode {
    #[serde(rename = "vhdl")]
    Vhdl,
    #[serde(rename = "verilog")]
    Verilog,
    #[serde(rename = "mixed")]
    Mixed
}

impl LangMode {
    pub fn supports_vhdl(&self) -> bool {
        match self {
            Self::Vhdl | Self::Mixed => true,
            _ => false
        }
    }

    pub fn supports_verilog(&self) -> bool {
        match self {
            Self::Verilog | Self::Mixed => true,
            _ => false
        }
    }
}

impl Default for LangMode {
    fn default() -> Self {
        Self::Mixed
    }
}


#[derive(Debug, PartialEq)]
pub enum Lang {
    Vhdl,
    Verilog,
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Vhdl => "vhdl",
            Self::Verilog => "verilog",
        })
    }
}

pub trait Code {
    fn get_source_code_file(&self) -> &str;
    fn get_symbol(&self) -> Option<&vhdl::symbol::VHDLSymbol>;
}

#[derive(Debug, PartialEq)]
pub enum LangUnit {
    Vhdl(PrimaryUnit),
    Verilog(String)
}

// impl Code for LangUnit {
//     fn get_source_code_file(&self) -> &str {
//         match &self {
//             Self::Vhdl(u) => u.get_unit().get_source_code_file(),
//             Self::Verilog(u) => u.as_ref(),
//         }
//     }

//     fn get_symbol(&self) -> Option<&vhdl::symbol::VHDLSymbol> {
//         match &self {
//             Self::Vhdl(u) => u.get_unit().get_symbol(),
//             Self::Verilog(u) => None,
//         }
//     }
// }

impl LangUnit {
    /// References the unit's identifier.
    pub fn get_name(&self) -> LangIdentifier {
        match &self {
            Self::Vhdl(u) => LangIdentifier::Vhdl(u.get_iden().clone()),
            Self::Verilog(u) => LangIdentifier::Verilog(u.clone()),
        }
    }

    /// Denotes the HDL language that is used for this unit.
    pub fn get_lang(&self) -> Lang {
        match &self {
            Self::Vhdl(_) => Lang::Vhdl,
            Self::Verilog(_) => Lang::Verilog,
        }
    }

    pub fn get_source_code_file(&self) -> &str {
        match &self {
            Self::Vhdl(u) => u.get_unit().get_source_code_file(),
            Self::Verilog(u) => u.as_ref(),
        }
    }

    pub fn get_symbol(&self) -> Option<&vhdl::symbol::VHDLSymbol> {
        match &self {
            Self::Vhdl(u) => u.get_unit().get_symbol(),
            Self::Verilog(_u) => None,
        }
    }
  
    /// Serializes the data into a toml inline table
    pub fn to_toml(&self) -> toml_edit::Value {
        let mut item = toml_edit::Value::InlineTable(InlineTable::new());
        let tbl = item.as_inline_table_mut().unwrap();
        tbl.insert("language", toml_edit::value(&self.get_lang().to_string()).into_value().unwrap());
        tbl.insert(
            "identifier",
            toml_edit::value(&self.get_name().to_string())
                .into_value()
                .unwrap(),
        );
        tbl.insert(
            "type",
            toml_edit::value(&self.to_string()).into_value().unwrap(),
        );
        item
    }

    /// Deserializes the data from a toml inline table.
    pub fn from_toml(tbl: &toml_edit::InlineTable) -> Option<Self> {
        let entry = tbl.get("language")?.as_str()?;
        match entry {
            "vhdl" => {
                Some(Self::Vhdl(PrimaryUnit::from_toml(tbl)?))
            },
            "verilog" => {
                Some(Self::Verilog(String::new()))
            },
            _ => panic!("unknown entry in serialized toml table {}", entry)
        }
    }
}

impl FromStr for LangIdentifier {
    type Err = vhdl::token::identifier::IdentifierError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Vhdl(VhdlIdentifier::from_str(&s)?))
    }
}

impl Display for LangUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Vhdl(u) => write!(f, "{}", u),
            Self::Verilog(u) => write!(f, "{}", u),
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum LangIdentifier {
    Vhdl(VhdlIdentifier),
    Verilog(String)
}

impl LangIdentifier {
    pub fn as_vhdl_id(&self) -> Option<&VhdlIdentifier> {
        match &self {
            Self::Vhdl(name) => Some(name),
            _ => None,
        }
    }
}

impl Display for LangIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Vhdl(u) => write!(f, "{}", u),
            Self::Verilog(u) => write!(f, "{}", u),
        }
    }
}

pub fn collect_units(files: &Vec<String>, lang_mode: &LangMode) -> Result<HashMap<LangIdentifier, LangUnit>, Fault> {
    // collect the VHDL units
    let vhdl_units = match lang_mode.supports_vhdl() {
        true => vhdl::primaryunit::collect_units(&files)?,
        false => HashMap::new(),
    };

    // collect the Verilog units
    let verilog_units = match lang_mode.supports_verilog() {
        true => verilog::primaryunit::collect_units(&files)?,
        false => HashMap::new(),
    };
   
    // merge the two results into a common struct
    let mut results = HashMap::with_capacity(vhdl_units.len() + verilog_units.len());
    for (k, v) in vhdl_units {
        results.insert(LangIdentifier::Vhdl(k), LangUnit::Vhdl(v));
    }
    for (k, v) in verilog_units {
        results.insert(LangIdentifier::Verilog(k), LangUnit::Verilog(v));
    }
    
    Ok(results)
}