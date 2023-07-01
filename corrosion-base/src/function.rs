use crate::Value;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct FunctionRef(pub(crate) u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct VariableRef(pub(crate) u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ImmediateRef(pub(crate) u32);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct BlockRef(pub(crate) u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
pub enum Type{
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

impl Type{
    pub fn is_integer(self) -> bool{
        match self {
            Self::I8 | Self::I16 | Self::I32 | Self::I64 => true,
            _ => false
        }
    }

    pub fn is_pointer(self) -> bool{
        if let Type::Ptr = self{
            true
        }
        else{
            false
        }
    }

    pub fn is_float(self) -> bool{
        !(self.is_integer() || self.is_float())
    }
}

#[derive(Debug)]
pub struct Function{
    pub id: FunctionRef,
    pub name: String,
    pub inputs: Vec<Type>,
    pub locals: Vec<(VariableRef, Type)>,
    pub outputs: Vec<Type>,
    pub entry: BlockRef,
    pub blocks: Vec<Block>
}

impl Function{
    pub(crate) fn new(id: FunctionRef) -> Self{
        Self{
            name: String::with_capacity(0),
            blocks: Vec::new(),
            entry: BlockRef(0),
            id,
            inputs: Vec::new(),
            locals: Vec::new(),
            outputs: Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct Block{
    pub label: BlockRef,
    pub inputs: Vec<Value>,
    pub instructions: Vec<Instruction>
}

impl Block{
    pub fn new(id: BlockRef) -> Self{
        Self{
            instructions: Vec::new(),
            inputs: Vec::new(),
            label: id
        }
    }
}

#[derive(Debug)]
pub enum Operation{
    
    ///
    /// Load an immediate 64 bit integer
    ConstI32(u32),

    ///
    /// Load an immediate 64 bit integer
    ConstI64(u64),

    ///
    /// Load an immedate 32 bit float
    ConstF32(f32),

    ///
    /// Load an immediate 64 bit float
    ConstF64(f64),
    
    ///
    /// Offset a pointer with an alignment of 1
    OffsetPtr1(ImmediateRef, ImmediateRef),

    ///
    /// Offset a pointer with an alignment of 2
    OffsetPtr2(ImmediateRef, ImmediateRef),

    ///
    /// Offset a pointer with an alignment of 4
    OffsetPtr4(ImmediateRef, ImmediateRef),

    ///
    /// Offset a pointer with an alignment of 8
    OffsetPtr8(ImmediateRef, ImmediateRef),

    ///
    /// Add two numbers of type
    Add(ImmediateRef, ImmediateRef, Type),

    ///
    /// Subtract two numbers of type
    Sub(ImmediateRef, ImmediateRef, Type),

    ///
    /// Multiply two numbers of type
    Mul(ImmediateRef, ImmediateRef, Type),

    ///
    /// Divide two numbers of type
    Div(ImmediateRef, ImmediateRef, Type),

    ///
    /// Subtract two numbers of type
    Mod(ImmediateRef, ImmediateRef, Type),
    
    ///
    /// Load a value from alocal variable
    LoadLocal(VariableRef),

    ///
    /// Write a value to a local variable
    StoreLocal(VariableRef, ImmediateRef),

    ///
    /// Read a 8 bit integer from memory
    ReadI8(ImmediateRef),

    ///
    /// Read a 16 bit integer from memory
    ReadI16(ImmediateRef),

    ///
    /// Read a 32 bit integer from memory
    ReadI32(ImmediateRef),

    ///
    /// Read a 64 bit integer from memory
    ReadI64(ImmediateRef),

    ///
    /// Read a 32 bit float from memory
    ReadF32(ImmediateRef),

    ///
    /// Read a 64 bit float from memory
    ReadF64(ImmediateRef),

    ///
    /// Writes a 8 bit integer from memory
    WriteI8(ImmediateRef),

    ///
    /// Writes a 16 bit integer from memory
    WriteI16(ImmediateRef),

    ///
    /// Writes a 32 bit integer from memory
    WriteI32(ImmediateRef),

    ///
    /// Writes a 64 bit integer from memory
    WriteI64(ImmediateRef),

    ///
    /// Writes a 32 bit float from memory
    WriteF32(ImmediateRef),

    ///
    /// Writes a 64 bit float from memory
    WriteF64(ImmediateRef),


    ///
    /// Branch to the first block if the immediate is == 0; otherwise the second block
    BranchIfEq(ImmediateRef, BlockRef, BlockRef),

    ///
    /// Branch to the first block if the immediate is != 0; otherwise the second block
    BranchIfNe(ImmediateRef, BlockRef, BlockRef),

    ///
    /// Branch to the first block if the immediate is < 0; otherwise the second block
    BranchIfLt(ImmediateRef, BlockRef, BlockRef),

    ///
    /// Branch to the first block if the immediate is <= 0; otherwise the second block
    BranchIfLe(ImmediateRef, BlockRef, BlockRef),

    ///
    /// Branch to the first block if the immediate is > 0; otherwise the second block
    BranchIfGt(ImmediateRef, BlockRef, BlockRef),
    
    ///
    /// Branch to the first block if the immediate is >= 0; otherwise the second block
    BranchIfGe(ImmediateRef, BlockRef, BlockRef),

    ///
    /// Branch to the block
    Branch(BlockRef),

    ///
    /// Return with the provided amount of immediates, must match the functions return values
    Return(Vec<ImmediateRef>),

    Invoke(FunctionRef, Vec<ImmediateRef>)
}

#[derive(Debug)]
pub struct Instruction{
    pub(crate) operation: Operation,
    pub(crate) output: Vec<ImmediateRef>,
}

impl Instruction{
    pub fn operation(&self) -> &Operation{
        &self.operation
    }

    pub fn immediates(&self) -> &[ImmediateRef]{
        &self.output
    }

    pub fn assert_1_immediate(&self) -> ImmediateRef{
        assert!(self.immediates().len() == 1, "Expected only 1 immediate, not {}",self.immediates().len());
        self.immediates()[0]
    }

    pub fn assert_2_immediates(&self) -> (ImmediateRef, ImmediateRef){
        assert!(self.immediates().len() == 2, "Expected exactly 2 immediates, not {}",self.immediates().len());
        (self.immediates()[0], self.immediates()[1])
    }
}