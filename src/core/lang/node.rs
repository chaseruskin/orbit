use colored::Colorize;
use crate::core::lang::vhdl::token::Identifier;

#[derive(Debug, PartialEq)]
pub struct HdlNode<'a> {
    sym: VHDLSymbol,
    files: Vec<&'a IpFileNode<'a>>, // must use a vector to retain file order in blueprint
}

impl<'a> HdlNode<'a> {
    pub fn new(sym: VHDLSymbol, file: &'a IpFileNode) -> Self {
        let mut set = Vec::with_capacity(1);
        set.push(file);
        Self {
            sym: sym,
            files: set,
        }
    }

    pub fn add_file(&mut self, ipf: &'a IpFileNode<'a>) {
        if self.files.contains(&ipf) == false {
            self.files.push(ipf);
        }
    }

    /// References the VHDL symbol
    pub fn get_symbol(&self) -> &VHDLSymbol {
        &self.sym
    }

    pub fn get_symbol_mut(&mut self) -> &mut VHDLSymbol {
        &mut self.sym
    }

    pub fn get_associated_files(&self) -> &Vec<&'a IpFileNode<'a>> {
        &self.files
    }

    pub fn is_black_box(&self) -> bool {
        self.files.is_empty()
    }

    pub fn black_box(sym: VHDLSymbol) -> Self {
        Self { sym: sym, files: Vec::new() }
    }

    pub fn display(&self, fmt: &IdentifierFormat) -> String {
        let name = self.sym.as_iden().unwrap_or(&Identifier::new()).to_string();
        if self.is_black_box() == true {
            format!("{} {}", &name.yellow(), "?".yellow())
        } else {
            match fmt {
                IdentifierFormat::Long => {
                    let ip = self.files.first().unwrap().get_ip();
                    format!("{} [{}]", &name, ip.get_man().get_ip().into_ip_spec())
                }
                IdentifierFormat::Short => format!("{}", &name),
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SubUnitNode<'a> {
    sub: SubUnit,
    file: &'a IpFileNode<'a>,
}

impl<'a> SubUnitNode<'a> {
    pub fn new(unit: SubUnit, file: &'a IpFileNode<'a>) -> Self {
        Self { sub: unit, file: file }
    }

    /// References the architecture struct.
    pub fn get_sub(&self) -> &SubUnit {
        &self.sub
    }

    /// References the ip file node.
    pub fn get_file(&self) -> &'a IpFileNode<'a> {
        &self.file
    }
}

use crate::core::lang::vhdl::subunit::SubUnit;
use crate::core::v2::algo::IpFileNode;
use crate::util::anyerror::AnyError;
use crate::core::lang::vhdl::symbol::VHDLSymbol;

#[derive(Debug, PartialEq)]
pub enum IdentifierFormat {
    Long,
    Short
}

impl std::str::FromStr for IdentifierFormat {
    type Err = AnyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            _ => Err(AnyError(format!("format can be 'long' or 'short'")))
        }
    }
}