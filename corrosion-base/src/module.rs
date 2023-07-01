use crate::{Function, FunctionRef};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ExportRef(pub(crate) u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ModuleRef(pub(crate) u32);


#[derive(Debug)]
pub struct Module{
    pub id: ModuleRef,
    pub functions: Vec<Function>,
    pub exports: Vec<ExportRef>,
}

impl Module{
    pub fn new(id: ModuleRef) -> Self{
        Self{
            exports: Vec::new(),
            functions: Vec::new(),
            id
        }
    }
}

impl From<FunctionRef> for ExportRef{
    fn from(value: FunctionRef) -> Self {
        ExportRef(value.0)
    }
}