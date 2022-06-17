use super::token::Identifier;

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

enum PortDirection { 
    In, 
    Out,
    Inout, 
    Linkage, 
    Buffer, 
}