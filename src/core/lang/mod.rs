pub mod sv;
pub mod verilog;
pub mod vhdl;

pub mod lexer;
pub mod parser;

pub mod node;
pub mod unit;

pub mod reference;

use crate::error::Error;
use crate::error::Hint;
use crate::util::anyerror::AnyError;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use toml_edit::InlineTable;
use verilog::symbols::VerilogSymbol;
use vhdl::symbols::VhdlSymbol;

type VhdlIdentifier = vhdl::token::Identifier;
type VerilogIdentifier = verilog::token::identifier::Identifier;

type VhdlPrimaryUnit = vhdl::primaryunit::PrimaryUnit;
type VerilogPrimaryUnit = verilog::primaryunit::PrimaryUnit;

use super::pubfile::{PublicList, Visibility};

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Language {
    vhdl: Option<bool>,
    verilog: Option<bool>,
    systemverilog: Option<bool>,
}

impl Language {
    pub fn with(vhdl: bool, verilog: bool, systemverilog: bool) -> Self {
        Self {
            vhdl: Some(vhdl),
            verilog: Some(verilog),
            systemverilog: Some(systemverilog),
        }
    }

    pub fn new() -> Self {
        Self {
            vhdl: None,
            verilog: None,
            systemverilog: None,
        }
    }

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) {
        if let Some(rhs) = rhs {
            // no build dir defined so give it the value from `rhs`
            if self.vhdl.is_some() == false {
                self.vhdl = rhs.vhdl;
            }
            if self.verilog.is_some() == false {
                self.verilog = rhs.verilog;
            }
            if self.systemverilog.is_some() == false {
                self.systemverilog = rhs.systemverilog;
            }
        }
    }

    pub fn supports_vhdl(&self) -> bool {
        self.vhdl.unwrap_or(true)
    }

    pub fn supports_verilog(&self) -> bool {
        self.verilog.unwrap_or(true)
    }

    pub fn supports_systemverilog(&self) -> bool {
        self.systemverilog.unwrap_or(true)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self {
            vhdl: None,
            verilog: None,
            systemverilog: None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Lang {
    #[serde(rename = "vhdl")]
    Vhdl,
    #[serde(rename = "verilog")]
    Verilog,
    #[serde(rename = "systemverilog")]
    SystemVerilog,
}

impl FromStr for Lang {
    type Err = AnyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vhdl" => Ok(Self::Vhdl),
            "verilog" => Ok(Self::Verilog),
            "systemverilog" => Ok(Self::SystemVerilog),
            _ => Err(AnyError(format!("unsupported language {:?}", s))),
        }
    }
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Vhdl => "vhdl",
                Self::Verilog => "verilog",
                Self::SystemVerilog => "systemverilog",
            }
        )
    }
}

pub trait Code {
    fn get_source_code_file(&self) -> &str;
    fn get_symbol(&self) -> Option<&vhdl::symbols::VhdlSymbol>;
}

#[derive(Debug, PartialEq)]
pub struct SharedData {
    visibility: Visibility,
}

impl SharedData {
    pub fn new() -> Self {
        Self {
            visibility: Visibility::default(),
        }
    }

    pub fn set_visibility(&mut self, v: Visibility) {
        self.visibility = v;
    }

    pub fn get_visibility(&self) -> &Visibility {
        &self.visibility
    }
}

#[derive(Debug, PartialEq)]
pub enum LangUnit {
    Vhdl(VhdlPrimaryUnit, SharedData),
    Verilog(VerilogPrimaryUnit, SharedData),
}

impl LangUnit {
    pub fn is_usable_component(&self) -> bool {
        match &self {
            Self::Verilog(m, _) => m.get_unit().is_usable_component().is_some(),
            Self::Vhdl(m, _) => m.get_unit().is_usable_component().is_some(),
        }
    }

    /// Checks if the module is public.
    pub fn is_listed_public(&self, plist: &PublicList) -> bool {
        plist.is_included(self.get_source_file())
    }

    pub fn get_visibility(&self) -> &Visibility {
        match &self {
            Self::Vhdl(_, sd) => sd.get_visibility(),
            Self::Verilog(_, sd) => sd.get_visibility(),
        }
    }

    pub fn set_visibility(&mut self, v: Visibility) {
        match self {
            Self::Vhdl(_, sd) => sd.set_visibility(v),
            Self::Verilog(_, sd) => sd.set_visibility(v),
        };
    }

    /// References the unit's identifier.
    pub fn get_name(&self) -> LangIdentifier {
        match &self {
            Self::Vhdl(u, _) => LangIdentifier::Vhdl(u.get_iden().clone()),
            Self::Verilog(u, _) => LangIdentifier::Verilog(u.get_name().clone()),
        }
    }

    /// Denotes the HDL language that is used for this unit.
    pub fn get_lang(&self) -> Lang {
        match &self {
            Self::Vhdl(_, _) => Lang::Vhdl,
            Self::Verilog(_, _) => Lang::Verilog,
        }
    }

    pub fn get_source_file(&self) -> &str {
        match &self {
            Self::Vhdl(u, _) => u.get_unit().get_source_file(),
            Self::Verilog(u, _) => u.get_unit().get_source_file(),
        }
    }

    pub fn get_vhdl_symbol(&self) -> Option<&VhdlSymbol> {
        match &self {
            Self::Vhdl(u, _) => u.get_unit().get_symbol(),
            Self::Verilog(_, _) => None,
        }
    }

    pub fn get_verilog_symbol(&self) -> Option<&VerilogSymbol> {
        match &self {
            Self::Vhdl(_, _) => None,
            Self::Verilog(u, _) => u.get_unit().get_symbol(),
        }
    }

    pub fn get_references(&self) -> Vec<LangIdentifier> {
        match &self {
            Self::Vhdl(u, _) => match u.get_unit().get_symbol() {
                Some(sym) => sym
                    .get_refs()
                    .into_iter()
                    .map(|f| f.get_suffix().clone())
                    .collect(),
                None => Vec::new(),
            },
            Self::Verilog(u, _) => match u.get_unit().get_symbol() {
                Some(sym) => sym
                    .get_refs()
                    .into_iter()
                    .map(|f| f.get_suffix().clone())
                    .collect(),
                None => Vec::new(),
            },
        }
    }

    /// Serializes the data into a toml inline table
    pub fn to_toml(&self) -> toml_edit::Value {
        let mut item = toml_edit::Value::InlineTable(InlineTable::new());
        let tbl = item.as_inline_table_mut().unwrap();
        tbl.insert(
            "language",
            toml_edit::value(&self.get_lang().to_string())
                .into_value()
                .unwrap(),
        );
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
            "vhdl" => Some(Self::Vhdl(
                VhdlPrimaryUnit::from_toml(tbl)?,
                SharedData::new(),
            )),
            "verilog" => Some(Self::Verilog(
                VerilogPrimaryUnit::from_toml(tbl)?,
                SharedData::new(),
            )),
            _ => panic!("unknown entry in serialized toml table {}", entry),
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
            Self::Vhdl(u, _) => write!(f, "{}", u),
            Self::Verilog(u, _) => write!(f, "{}", u),
        }
    }
}

#[derive(Debug, Eq, Clone, PartialOrd, Ord)]
pub enum LangIdentifier {
    Vhdl(VhdlIdentifier),
    Verilog(VerilogIdentifier),
}

impl PartialEq for LangIdentifier {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            Self::Vhdl(l_vhdl_name) => match &other {
                Self::Vhdl(r_vhdl_name) => l_vhdl_name == r_vhdl_name,
                Self::Verilog(r_verilog_name) => l_vhdl_name.as_str() == r_verilog_name.as_str(),
            },
            Self::Verilog(l_verilog_name) => match &other {
                Self::Verilog(r_verilog_name) => l_verilog_name == r_verilog_name,
                Self::Vhdl(r_vhdl_name) => l_verilog_name.as_str() == r_vhdl_name.as_str(),
            },
        }
    }
}

impl Hash for LangIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl From<VhdlIdentifier> for LangIdentifier {
    fn from(value: VhdlIdentifier) -> Self {
        Self::Vhdl(value)
    }
}

impl From<VerilogIdentifier> for LangIdentifier {
    fn from(value: VerilogIdentifier) -> Self {
        Self::Verilog(value)
    }
}

impl LangIdentifier {
    pub fn new_working() -> Self {
        Self::Vhdl(VhdlIdentifier::new_working())
    }

    pub fn as_vhdl_name(&self) -> Option<&VhdlIdentifier> {
        match &self {
            Self::Vhdl(name) => Some(name),
            _ => None,
        }
    }

    pub fn as_verilog_name(&self) -> Option<&VerilogIdentifier> {
        match &self {
            Self::Verilog(name) => Some(name),
            _ => None,
        }
    }

    fn as_str(&self) -> &str {
        match &self {
            Self::Verilog(name) => name.as_str(),
            Self::Vhdl(name) => name.as_str(),
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

pub fn collect_units(
    files: &Vec<String>,
    lang_mode: &Language,
) -> Result<HashMap<LangIdentifier, LangUnit>, Box<dyn std::error::Error>> {
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
        results.insert(
            LangIdentifier::Vhdl(k),
            LangUnit::Vhdl(v, SharedData::new()),
        );
    }
    for (k, v) in verilog_units {
        let source_file = v.get_unit().get_source_file().to_string();
        let existing = results.insert(
            LangIdentifier::Verilog(k),
            LangUnit::Verilog(v, SharedData::new()),
        );
        if let Some(existing_unit) = existing {
            // return duplicate id error
            return Err(Error::DuplicateIdentifiersCrossLang(
                existing_unit.get_name().to_string(),
                existing_unit.get_source_file().to_string(),
                source_file,
                Hint::ResolveDuplicateIds1,
            ))?;
        }
    }
    Ok(results)
}
