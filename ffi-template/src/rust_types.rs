use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MetaType {
    Func(RustFunction),
    Struct(RustStructure),
    FreeFunc(RustFreeFunction),
}

impl From<RustFunction> for MetaType {
    fn from (s: RustFunction) -> MetaType {
        MetaType::Func(s)
    }
}

impl From<RustStructure> for MetaType {
    fn from (s: RustStructure) -> MetaType {
        MetaType::Struct(s)
    }
}

impl From<RustFreeFunction> for MetaType {
    fn from (s: RustFreeFunction) -> MetaType {
        MetaType::FreeFunc(s)
    }
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct RustFreeFunction<> {
    pub ty: RustTypes,
    pub func: RustFunction,
}


#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct RustStructure {
    pub self_ty: String,
    pub methods: Vec<RustFunction>,
    pub destructor: Option<String>
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct RustFunction {
    pub name: String,
    pub extern_name: String,
    pub inputs: Vec<RustArgument>,
    pub output: Option<RustTypes>,
}

impl AsRef<RustFunction> for RustFunction {
    fn as_ref(&self) -> &RustFunction {
        self
    }
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
pub struct RustArgument {
    pub name: String,
    pub ty: RustTypes,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RustTypes {
    Ptr(String),
    Option(Box<RustTypes>),
    Result(Box<RustTypes>),
    Primitive(String),
    String,
}

impl RustTypes {
    pub fn get_root(&self) -> &RustTypes {
        match self {
            RustTypes::Ptr(_) => self,
            RustTypes::Option(s) => RustTypes::get_root(s.as_ref().get_root()),
            RustTypes::Result(s) => RustTypes::get_root(s.as_ref().get_root()),
            RustTypes::Primitive(_) => self,
            RustTypes::String => self,
        }
    }
}

impl AsRef<RustTypes> for RustTypes {
    fn as_ref(&self) -> &RustTypes {
        self
    }
}

impl Default for RustTypes {
    fn default() -> RustTypes {
        RustTypes::Ptr("c_void".to_owned())
    }
}

impl ToString for RustTypes {
    fn to_string(&self) -> String {
        match self {
            RustTypes::Ptr(s) => format!("*mut {}", s),
            RustTypes::Option(s) => format!("Option<{}>", s.to_string()),
            RustTypes::Result(s) => format!("Result<{}>", s.to_string()),
            RustTypes::Primitive(s) => s.clone(),
            RustTypes::String => "String".to_owned(),
        }
    }
}
