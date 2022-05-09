use super::vhdl::Identifier;

struct Library {
    name: Identifier,
}

struct Entity {
    imports: Vec<Library>, 
    name: Identifier,
    architectures: Vec<Architecture>,
    ports: Vec<Port>,
    generics: Vec<Generic>,
}

impl Entity {
    /// Checks if the port list is empty for this entity.
    fn is_testbench(&self) -> bool {
        self.ports.is_empty()
    }
}

struct Generic {
    name: Identifier,
    dtype: Identifier,
    value: Option<String>,
}

struct Port {
    name: Identifier,
    direction: PortDirection,
    dtype: Identifier,
    value: Option<String>,
}

enum PortDirection { In, Out,Inout, Linkage, Buffer, }

struct Architecture {
    imports: Vec<Library>, // architecture can have its own set of imports, but also inherit from entity
    name: Identifier,
}