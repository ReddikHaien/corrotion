use crate::{FunctionRef, Type, Function, Module, ModuleRef, Block, BlockRef, ImmediateRef, Instruction, Operation, VariableRef};

pub struct ModuleBuilder{
    functions: Vec<Function>,
}

impl ModuleBuilder{

    pub fn new() -> Self{
        Self{
            functions: Vec::new()
        }
    }

    pub fn new_function(&mut self) -> FunctionRef{
        let id = FunctionRef((self.functions.len()+1) as u32);

        self.functions.push(Function::new(id));

        id
    }

    pub fn funtion_builder<'a>(&'a mut self, function: FunctionRef) -> FunctionBuilder<'a>{
        FunctionBuilder{
            function,
            module: self,
            immediate_counter: 0
        }
    }

    pub fn build(self) -> Module{
        Module{
            id: ModuleRef(0),
            functions: self.functions,
            exports: vec![]
        }
    }
}

pub struct FunctionBuilder<'a>{
    module: &'a mut ModuleBuilder,
    function: FunctionRef,
    immediate_counter: u32,
}

impl<'a> FunctionBuilder<'a>{
    pub fn new(function: FunctionRef, module: &'a mut ModuleBuilder) -> FunctionBuilder<'_> {
        Self{
            function,
            module,
            immediate_counter: 0,
        }
    }

    fn function(&self) -> &Function{
        &self.module.functions[(self.function.0 - 1) as usize]
    }


    fn function_mut(&mut self) -> &mut Function{
        &mut self.module.functions[(self.function.0 - 1) as usize]
    }

    pub fn add_input(&mut self, type_: Type){
        self.function_mut().inputs.push(type_);
    }

    pub fn add_output(&mut self, type_: Type){
        self.function_mut().outputs.push(type_);
    }

    pub fn add_local(&mut self, type_: Type) -> VariableRef{
        let id = VariableRef((self.function().locals.len()+1) as u32);
        self.function_mut().locals.push((id, type_));
        id
    }

    pub fn create_block(&mut self) -> BlockRef{
        let id = BlockRef((self.function().blocks.len()+1) as u32);
        self.function_mut().blocks.push(Block::new(id));
        id
    }

    pub fn block_builder(&'a mut self, block: BlockRef) -> BlockBuilder<'_>{
        BlockBuilder{
            fb: self,
            cur_block: block
        }
    }
}


pub struct BlockBuilder<'a>{
    fb: &'a mut FunctionBuilder<'a>,
    cur_block: BlockRef
}

impl<'a> BlockBuilder<'a>{

    fn new_immediate(&mut self) -> ImmediateRef{
        self.fb.immediate_counter += 1;
        ImmediateRef(self.fb.immediate_counter)
    }

    fn block(&self) -> &Block{
        &self.fb.function().blocks[(self.cur_block.0 - 1) as usize]
    }

    fn block_mut(&mut self) -> &mut Block{
        &mut self.fb.function_mut().blocks[(self.cur_block.0 - 1) as usize]
    }

    pub fn into_entry_block(&mut self){
        let mut args = Vec::with_capacity(self.fb.function().inputs.len());

        for i in 0..args.capacity(){
            let imm = self.new_immediate();
            
            let type_ = self.fb.function().inputs[i];
            args.push(Value(imm, type_));
        }

        let b = self.block_mut();
        b.inputs = args;
        self.fb.function_mut().entry = b.label;
    }

    pub fn add_param(&mut self, type_: Type)-> Value{
        let output = self.new_immediate();
        self.block_mut().inputs.push(Value(output, type_));
        Value(output, type_)
    }

    pub fn get_params(&self) -> &[Value]{
        &self.block().inputs
    }

    pub fn const_f32(&mut self, value: f32) -> Value{
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::ConstF32(value),
            output: vec![output]
        });

        Value(output, Type::F32)
    }

    pub fn const_f64(&mut self, value: f64) -> Value{
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::ConstF64(value),
            output: vec![output]
        });

        Value(output, Type::F64)
    }

    pub fn const_i32(&mut self, value: i32) -> Value{
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::ConstI32(value as u32),
            output: vec![output]
        });
        Value(output, Type::I32)
    }

    pub fn const_i64(&mut self, value: i64) -> Value{
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::ConstI64(value as u64),
            output: vec![output]
        });
        Value(output, Type::I64)
    }

    pub fn add_values(&mut self, a: Value, b: Value) -> Value{
        assert!(a.1 == b.1 && !a.1.is_pointer(), "expects {:?} and {:?} to be the same numeric type", a.1, b.1);
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::Add(a.0, b.0, a.1),
            output: vec![output]
        });

        Value(output, a.1)
    }

    pub fn sub_values(&mut self, a: Value, b: Value) -> Value{
        assert!(a.1 == b.1 && !a.1.is_pointer(), "expects {:?} and {:?} to be the same numeric type", a.1, b.1);
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::Sub(a.0, b.0, a.1),
            output: vec![output]
        });

        Value(output, a.1)
    }

    pub fn mul_values(&mut self, a: Value, b: Value) -> Value{
        assert!(a.1 == b.1 && !a.1.is_pointer(), "expects {:?} and {:?} to be the same numeric type", a.1, b.1);
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::Mul(a.0, b.0, a.1),
            output: vec![output]
        });

        Value(output, a.1)
    }

    pub fn div_values(&mut self, a: Value, b: Value) -> Value{
        assert!(a.1 == b.1 && !a.1.is_pointer(), "expects {:?} and {:?} to be the same numeric type", a.1, b.1);
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::Div(a.0, b.0, a.1),
            output: vec![output]
        });

        Value(output, a.1)
    }

    pub fn modulus_values(&mut self, a: Value, b: Value) -> Value{
        assert!(a.1 == b.1 && !a.1.is_pointer(), "expects {:?} and {:?} to be the same numeric type", a.1, b.1);
        let output = self.new_immediate();
        self.block_mut().instructions.push(Instruction{
            operation: Operation::Mod(a.0, b.0, a.1),
            output: vec![output]
        });

        Value(output, a.1)
    }

    pub fn get_local(&mut self, var: VariableRef) -> Value{
        let output = self.new_immediate();

        let type_ = self.fb.function().locals.iter().find(|x| x.0 == var).expect("Variable to be defined").1;

        self.block_mut().instructions.push(Instruction{
            operation: Operation::LoadLocal(var),
            output: vec![output]
        });

        Value(output, type_)
    }

    pub fn set_local(&mut self, var: VariableRef, value: Value){
        let type_ = self.fb.function().locals.iter().find(|x| x.0 == var).expect("Variable to be defined").1;
        assert!(type_ == value.1, "{:?} and {:?} must be the same type", type_, value.1);
        self.block_mut().instructions.push(Instruction{
            operation: Operation::StoreLocal(var, value.0),
            output: Vec::with_capacity(0)
        });
    }

    pub fn return_(&mut self, values: &[Value]){

        assert!(values.len() == self.fb.function().outputs.len());

        assert!(values.iter().zip(self.fb.function().outputs.iter()).all(|(a, b)|a.1 == *b));

        self.block_mut().instructions.push(Instruction{
            operation: Operation::Return(values.iter().map(|x| x.0).collect::<Vec<_>>()),
            output: Vec::with_capacity(0)
        })
    }
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Value(ImmediateRef, Type);

