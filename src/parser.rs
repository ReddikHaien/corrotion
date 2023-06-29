pub mod methods;

use std::collections::HashMap;

use cranelift::prelude::*;

use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{default_libcall_names, Module, FuncId};

use crate::frontend::{Function, Expression, Statement};

use self::methods::{MethodResolver, register_builtin_methods};

#[derive(Clone, Copy, Debug)]
pub enum ValueType{
    Integer,
    F32,
    F64,
    Ptr
}

#[derive(Clone, Copy, Debug)]
pub enum ResolvedTypes{
    Block(Block),
    Value(Value, ValueType),

}

impl ResolvedTypes{
    pub fn as_value(self) -> (Value, ValueType){
        let Self::Value(value, ty) = self else {panic!("type is not a `Value`: {:?}", self)};
        (value, ty)
    }
}

pub struct CodeGenerator{
    module: JITModule,
    variable_builder: VariableBuilder,
    functions: HashMap<String, FunctionDefinition>,
    methods: HashMap<&'static str, Box<dyn MethodResolver>>
}

impl CodeGenerator{
    pub fn new() -> Self{
        let builder = JITBuilder::new(default_libcall_names()).unwrap();
        let mut map = HashMap::new();
        register_builtin_methods(&mut map);
        Self{
            module: JITModule::new(builder),
            variable_builder: VariableBuilder::new(),
            functions: HashMap::new(),
            methods: map
        }
    }


    pub fn function(&mut self, function: &Function){
        let sig = self.signature(function);


        if !self.functions.contains_key(function.name){
            self.functions.insert(function.name.to_owned(), FunctionDefinition {
                signature: sig.clone(),
                id: FuncId::from_u32(0)
            });
        }
        else{
            panic!("Duplicate definition of {}", function.name)
        }

        let id = self.module.declare_function(function.name, cranelift_module::Linkage::Export, &sig).unwrap();

        self.functions.get_mut(function.name).unwrap().id = id;
        
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        
        let mut blocks = Vec::new();
        let mut entry_statements = Vec::new();

        for statement in function.statements.iter(){
            if let Expression::Block { .. } = &statement.expression{
                blocks.push(statement);
            }
            else{
                entry_statements.push(statement);
            }
        }

        let mut variables = HashMap::new();

        let mut fctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fctx);

        let entry_block = builder.create_block();
        
        builder.switch_to_block(entry_block);
        
        builder.append_block_params_for_function_params(entry_block);
        for (index, (name, ty)) in function.params.iter().enumerate(){
            let var = self.variable_builder.get_var();
            let type_ = ty.into();
            builder.declare_var(Variable::new(var), type_);
            let value = builder.block_params(entry_block)[index];
            builder.def_var(Variable::new(var), value);
            variables.insert(*name, (var, type_));
        }

        let mut blocks_map = HashMap::new();
        for b in blocks.iter(){
            assert!(b.output.len() == 1, "blocks at root level must be assigned to a name");
            let name = b.output[0].to_owned();
            let block = builder.create_block();
            blocks_map.insert(name, block);
        }

        //Filling entry block
        self.block(&entry_statements, &mut builder, &mut variables, &mut blocks_map)


    }

    fn block(&mut self, statements: &[&Statement], builder: &mut FunctionBuilder, variables: &mut HashMap<&str, (usize, Type)>, parent_blocks: &mut HashMap<String, Block>){
        let immediates = HashMap::<&str, ResolvedTypes>::new();
        for statement in statements{
            let Expression::Method {..} = &statement.expression else {panic!("Only method expressions are allowed at statement level")};
            let values = self.expression(&statement.expression, builder, variables, parent_blocks);

        }
    }

    fn expression(&mut self, expression: &Expression, builder: &mut FunctionBuilder, variables: &mut HashMap<&str, (usize, Type)>, parent_blocks: &mut HashMap<String, Block>) -> Vec<ResolvedTypes>{

        match expression {
            Expression::Method { name, arguments } => {
                let mut args = Vec::new();
                for a in arguments.iter(){
                    args.extend(self.expression(a, builder, variables, parent_blocks));
                }

                todo!();
            },
            Expression::Numeric(_) => todo!(),
            Expression::Block { params, statements } => {
                let new = builder.create_block();
                let c = builder.current_block().unwrap();
                builder.switch_to_block(new);
                for (_, p) in params{
                    builder.append_block_param(new, p.into());
                }
                builder.switch_to_block(c);
                vec![ResolvedTypes::Block(new) ]
            },
            Expression::Identifier(n) => {
                let (i, ty) = variables.get(n).unwrap();
                let value = builder.use_var(Variable::new(*i));
                vec![ResolvedTypes::Value(value, ty.clone().into())]
            },
        }

    }


    pub fn signature(&self, function: &Function) -> Signature {
        let mut sig = self.module.make_signature();

        for (_, ty) in function.params.iter(){
            sig.params.push(AbiParam::new(ty.into()));
        }

        for ty in function.returns.iter(){
            sig.returns.push(AbiParam::new(ty.into()));
        }

        sig
    }
}

impl<'a> From<&crate::frontend::Type<'a>> for Type{
    fn from(value: &crate::frontend::Type<'a>) -> Self {
        match value{
            crate::frontend::Type::U8 | 
            crate::frontend::Type::I8 => types::I8,
            crate::frontend::Type::U16 |
            crate::frontend::Type::I16 => types::I16,
            crate::frontend::Type::U32 |
            crate::frontend::Type::I32 => types::I32,
            crate::frontend::Type::U64 |
            crate::frontend::Type::I64 => types::I64,
            crate::frontend::Type::F32 => types::F32,
            crate::frontend::Type::F64 => types::F64,
            crate::frontend::Type::Pointer(_) => types::R64
        }
    }
}

impl From<Type> for ValueType{
    fn from(value: Type) -> Self {
        if value.is_int(){
            Self::Integer
        }
        else if value.is_float(){
            match value.bits(){
                32 => Self::F32,
                64 => Self::F64,
                x => unreachable!("No float with bitsize {}",x)
            }
        }
        else if value.is_ref(){
            Self::Ptr
        }
        else{
            unimplemented!("Unsupported type {}",value)
        }
    }
}

struct FunctionDefinition{
    signature: Signature,
    id: FuncId
}

struct VariableBuilder{
    index: usize
}

impl VariableBuilder{
    pub fn new() -> Self{
        Self{
            index: 0
        }
    }

    pub fn get_var(&mut self) -> usize{
        self.index+=1;
        self.index
    }
}