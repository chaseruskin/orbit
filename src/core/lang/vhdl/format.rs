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

    /// Merges any populated data from `rhs` into attributes that do not already
    /// have data defined in `self`.
    pub fn merge(&mut self, rhs: Option<Self>) -> () {
        if let Some(rhs) = rhs {
            if self.tab_size.is_some() == false {
                self.tab_size = rhs.tab_size
            }
            if self.type_auto_alignment.is_some() == false {
                self.type_auto_alignment = rhs.type_auto_alignment
            }
            if self.type_offset.is_some() == false {
                self.type_offset = rhs.type_offset
            }
            if self.mapping_auto_alignment.is_some() == false {
                self.mapping_auto_alignment = rhs.mapping_auto_alignment
            }
            if self.mapping_offset.is_some() == false {
                self.mapping_offset = rhs.mapping_offset
            }
            if self.indent_interfaces.is_some() == false {
                self.indent_interfaces = rhs.indent_interfaces
            }
            if self.space_interface_parenthesis.is_some() == false {
                self.space_interface_parenthesis = rhs.space_interface_parenthesis
            }
            if self.instance_name.is_some() == false {
                self.instance_name = rhs.instance_name
            }
        }
    }
}