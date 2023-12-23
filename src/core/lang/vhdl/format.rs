use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VhdlFormat {
    #[serde(rename = "tab-size")]
    tab_size: Option<u8>,
    #[serde(rename = "type-auto-alignment")]
    type_auto_alignment: Option<bool>,
    #[serde(rename = "type-offset")]
    type_offset: Option<u8>,
    #[serde(rename = "mapping-auto-alignment")]
    mapping_auto_alignment: Option<bool>,
    #[serde(rename = "mapping-offset")]
    mapping_offset: Option<u8>,
    #[serde(rename = "indent-interface")]
    indent_interfaces: Option<bool>,
    #[serde(rename = "space-interface-parenthesis")]
    space_interface_parenthesis: Option<bool>,
    #[serde(rename = "instance-name")]
    instance_name: Option<String>,
}

impl VhdlFormat {
    pub fn new() -> Self {
        Self {
            tab_size: Some(2),
            type_auto_alignment: Some(true),
            type_offset: Some(1),
            mapping_auto_alignment: Some(true),
            mapping_offset: Some(1),
            indent_interfaces: Some(true),
            space_interface_parenthesis: Some(false),
            instance_name: Some(String::from("uX")),
        }
    }

    pub fn get_tab_size(&self) -> u8 {
        self.tab_size.unwrap_or(2)
    }

    pub fn is_auto_type_aligned(&self) -> bool {
        self.type_auto_alignment.unwrap_or(false)
    }

    pub fn get_type_offset(&self) -> u8 {
        self.type_offset.unwrap_or(1)
    }

    pub fn is_auto_mapping_aligned(&self) -> bool {
        self.mapping_auto_alignment.unwrap_or(true)
    }

    pub fn get_mapping_offset(&self) -> u8 {
        self.mapping_offset.unwrap_or(1)
    }

    pub fn is_indented_interfaces(&self) -> bool {
        self.indent_interfaces.unwrap_or(true)
    }

    pub fn is_interface_parenthesis_spaced(&self) -> bool {
        self.space_interface_parenthesis.unwrap_or(false)
    }

    pub fn get_instance_name(&self) -> String {
        self.instance_name.as_ref().unwrap_or(&String::from("uX")).clone()
    }

    /// Overwrites any existing values with the data present in `rhs`.
    pub fn merge(&mut self, rhs: Option<Self>) -> () {
        if let Some(rhs) = rhs {
            if rhs.tab_size.is_some() == true {
                self.tab_size = rhs.tab_size
            }
            if rhs.type_auto_alignment.is_some() == true {
                self.type_auto_alignment = rhs.type_auto_alignment
            }
            if rhs.type_offset.is_some() == true {
                self.type_offset = rhs.type_offset
            }
            if rhs.mapping_auto_alignment.is_some() == true {
                self.mapping_auto_alignment = rhs.mapping_auto_alignment
            }
            if rhs.mapping_offset.is_some() == true {
                self.mapping_offset = rhs.mapping_offset
            }
            if rhs.indent_interfaces.is_some() == true {
                self.indent_interfaces = rhs.indent_interfaces
            }
            if rhs.space_interface_parenthesis.is_some() == true {
                self.space_interface_parenthesis = rhs.space_interface_parenthesis
            }
            if rhs.instance_name.is_some() == true {
                self.instance_name = rhs.instance_name
            }
        }
    }
}