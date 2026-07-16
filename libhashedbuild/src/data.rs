use crate::scheme;

#[repr(C)]
pub enum Type {
    Integer,
    Float,
    String,
    File,
    Structure,
    Function,
}

#[repr(C)]
pub struct Value {
    t: Type,
    u: ValueU,
}

#[repr(C)]
pub union ValueU {
    int: i64,
    float: f64,
    string: LenStr,
    path: *const u8,
    structure: Structure,
    function: Function,
    throw: Throw,
    future: Future,
}

#[repr(C)]
pub struct LenStr {
    pub len: u64,
    pub bytes: *mut u8,
}

#[repr(C)]
pub enum FunctionType {
    Custom,
    StructureBuilder,
    Accessor,
    FunctionBuilder,
    Adder,
    Subtractor,
    Multiplier,
    Divider,
    Moduler,
}

#[repr(C)]
pub struct Function {
    t: FunctionType,
    u: FunctionU,
}

#[repr(C)]
pub union FunctionU {
    custom: scheme::Scheme,
}

#[repr(C)]
pub struct Structure {
    pub len: u64,
    pub cap: u64,
    pub fields: *mut Field,
}

#[repr(C)]
pub struct Field {
    pub name: *const u8,
    pub val: Type,
}
