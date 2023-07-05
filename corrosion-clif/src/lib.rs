use std::collections::HashMap;
use cranelift::prelude::{*, isa::x64::settings::builder};
use corrosion_base::{Module as CModule, Type as CType, ModuleRef, FunctionRef, Function};
use cranelift_jit::{JITModule, JITBuilder};
use cranelift_module::{default_libcall_names, FuncId, Module, Linkage};

pub struct Generator{
    module: JITModule,
    functions: HashMap<(ModuleRef, FunctionRef), FunctionDeclaration>,
    var_counter: u32,
}

impl Generator{
    pub fn new() -> Self{
        let mut builder = JITBuilder::new(default_libcall_names()).unwrap();
        builder.hotswap(true);
        Self{
            module: JITModule::new(builder),
            functions: HashMap::new(),
            var_counter: 0
        }
    }

    pub fn load_module(&mut self, module: CModule){
        let mut ctx = self.module.make_context();
        let mut f_ctx = FunctionBuilderContext::new();
        
        for function in &module.functions{
            let declaration = self.declare_function(&module, function);

            if declaration.defined{
                panic!("Duplicate definition of function ({:?} {:?}) {}({:?}) -> {:?}", module.id, function.id, function.name, function.inputs, function.outputs);
            }

            let id = declaration.id;
            ctx.func.signature = declaration.signature.clone();

            {
                let mut b_ctx = FunctionBuilder::new(&mut ctx.func, &mut f_ctx);

                let mut block_map = HashMap::with_capacity(function.blocks.len());
                for block in &function.blocks{
                    let block_id = b_ctx.create_block();
                    block_map.insert(block.label, block_id);
                }

                let mut locals = HashMap::new();

                for (var, type_) in &function.locals {
                    let var_ = Variable::from_u32(self.var_counter);
                    b_ctx.declare_var(var_, from_base_type(*type_));
                    locals.insert(*var, var_);
                }

                for b in &function.blocks{
                    let block = block_map[&b.label];
                    let mut values = HashMap::new();

                    b_ctx.switch_to_block(block);

                    //Initalize parameters
                    if b.label == function.entry{
                        assert!(b.inputs.len() == function.inputs.len());
                        b_ctx.append_block_params_for_function_params(block);
                        
                        for (a, b) in b.inputs.iter().zip(b_ctx.block_params(block).iter()){
                            let imm = a.immediate();
                            values.insert(imm, *b);
                        }
                    }
                    else{
                        for input in &b.inputs{
                            let value = b_ctx.append_block_param(block, from_base_type(input.type_()));
                            values.insert(input.immediate(), value);
                        }
                    }

                    for instruction in &b.instructions{
                        match instruction.operation(){
                            corrosion_base::Operation::ConstI32(value) => {
                                let output = instruction.assert_1_immediate();
                                let value = b_ctx.ins().iconst(types::I32, *value as i32 as i64);
                                values.insert(output, value);
                            },
                            corrosion_base::Operation::ConstI64(value) => {
                                let output = instruction.assert_1_immediate();
                                let value = b_ctx.ins().iconst(types::I64, *value as i64);
                                values.insert(output, value);
                            },
                            corrosion_base::Operation::ConstF32(value) => {
                                let output = instruction.assert_1_immediate();
                                let value = b_ctx.ins().f32const(*value);
                                values.insert(output, value);
                            },
                            corrosion_base::Operation::ConstF64(value) => {
                                let output = instruction.assert_1_immediate();
                                let value = b_ctx.ins().f64const(*value);
                                values.insert(output, value);

                            },
                            corrosion_base::Operation::OffsetPtr1(_, _) => todo!("offset ptr with alignment 1"),
                            corrosion_base::Operation::OffsetPtr2(_, _) => todo!("offset ptr with alignment 2"),
                            corrosion_base::Operation::OffsetPtr4(_, _) => todo!("offset ptr with alignment 4"),
                            corrosion_base::Operation::OffsetPtr8(_, _) => todo!("offset ptr with alignment 8"),
                            corrosion_base::Operation::Add(a, b, type_) => {
                                let a_v = values[a];
                                let b_v = values[b];
                                let value = match type_{
                                    CType::I8 | 
                                    CType::I16 | 
                                    CType::I32 | 
                                    CType::I64 => b_ctx.ins().iadd(a_v, b_v),
                                    CType::F32 |
                                    CType::F64 => b_ctx.ins().fadd(a_v, b_v),
                                    _ => panic!("Cannot add types of {:?}",type_)
                                };
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            },
                            corrosion_base::Operation::Sub(a, b, type_) => {
                                let a_v = values[a];
                                let b_v = values[b];
                                let value = match type_{
                                    CType::I8 | 
                                    CType::I16 | 
                                    CType::I32 | 
                                    CType::I64 => b_ctx.ins().isub(a_v, b_v),
                                    CType::F32 |
                                    CType::F64 => b_ctx.ins().fsub(a_v, b_v),
                                    _ => panic!("Cannot subtract types of {:?}",type_)
                                };
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            },

                            corrosion_base::Operation::Mul(a, b, type_) => {
                                let a_v = values[a];
                                let b_v = values[b];
                                let value = match type_{
                                    CType::I8 | 
                                    CType::I16 | 
                                    CType::I32 | 
                                    CType::I64 => b_ctx.ins().imul(a_v, b_v),
                                    CType::F32 |
                                    CType::F64 => b_ctx.ins().fmul(a_v, b_v),
                                    _ => panic!("Cannot multiply types of {:?}",type_)
                                };
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            }
                            corrosion_base::Operation::Div(a, b, type_) => {
                                let a_v = values[a];
                                let b_v = values[b];
                                let value = match type_{
                                    CType::I8 | 
                                    CType::I16 | 
                                    CType::I32 | 
                                    CType::I64 => b_ctx.ins().sdiv(a_v, b_v),
                                    CType::F32 |
                                    CType::F64 => b_ctx.ins().fdiv(a_v, b_v),
                                    _ => panic!("Cannot divide types of {:?}",type_)
                                };
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            }
                            corrosion_base::Operation::Mod(a, b, type_) => {
                                let a_v = values[a];
                                let b_v = values[b];
                                let value = match type_{
                                    CType::I8 | 
                                    CType::I16 | 
                                    CType::I32 | 
                                    CType::I64 => b_ctx.ins().srem(a_v, b_v),
                                    CType::F32 |
                                    CType::F64 => unimplemented!("Float remainder is not yet supported"),
                                    _ => panic!("Cannot take remainder of {:?}",type_)
                                };
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            }
                            corrosion_base::Operation::LoadLocal(var) => {
                                let var_ = locals.get(var).unwrap();
                                let value = b_ctx.use_var(*var_);
                                let output = instruction.assert_1_immediate();
                                values.insert(output, value);
                            },
                            corrosion_base::Operation::StoreLocal(var, value) => {
                                let var_ = locals.get(var).unwrap();
                                b_ctx.def_var(*var_, values[value]);
                                instruction.assert_no_immediates();
                            },
                            corrosion_base::Operation::Read(ptr, type_, aligned) => {
                                let ptr = values[ptr];
                                let output = instruction.assert_1_immediate();
                                
                                let mut flags = MemFlags::new().with_heap();
                                if *aligned{ flags.set_aligned(); }
                                let value = b_ctx.ins().load(from_base_type(*type_), flags, ptr, 0);

                                values.insert(output, value);
                            },
                            corrosion_base::Operation::Write(ptr, value, aligned) => {
                                let ptr = values[ptr];
                                let value = values[value];

                                let mut flags = MemFlags::new().with_heap();
                                if *aligned{ flags.set_aligned();}
                                b_ctx.ins().store(flags, ptr, value, 0);
                            },
                            corrosion_base::Operation::BranchIfEq(_, _, _) => todo!(),
                            corrosion_base::Operation::BranchIfNe(_, _, _) => todo!(),
                            corrosion_base::Operation::BranchIfLt(_, _, _) => todo!(),
                            corrosion_base::Operation::BranchIfLe(_, _, _) => todo!(),
                            corrosion_base::Operation::BranchIfGt(_, _, _) => todo!(),
                            corrosion_base::Operation::BranchIfGe(_, _, _) => todo!(),
                            corrosion_base::Operation::Branch(_) => todo!(),
                            corrosion_base::Operation::Return(outputs) => {
                                let outputs = outputs.iter().map(|x| values[x]).collect::<Vec<_>>();

                                b_ctx.ins().return_(&outputs);
                            },
                            corrosion_base::Operation::Invoke(_, _) => todo!(),
                        }
                    }
                }

                b_ctx.seal_all_blocks();
                b_ctx.finalize();
            }
            self.module.define_function(id, &mut ctx).unwrap();
            self.module.clear_context(&mut ctx);

            let decl = self.functions.get_mut(&(module.id, function.id)).unwrap();
            decl.defined = true;
        }

        self.module.finalize_definitions().unwrap();
    }

    fn declare_function(&mut self,module: &CModule, function: &Function) -> &FunctionDeclaration{
        let key = (module.id, function.id);

        match self.functions.entry(key) {
            std::collections::hash_map::Entry::Occupied(declaration) => {
                declaration.into_mut()
            },
            std::collections::hash_map::Entry::Vacant(entry) => {
                let mut sig = self.module.make_signature();
                for input in &function.inputs{
                    sig.params.push(AbiParam::new(from_base_type(*input)));
                }
                for output in &function.outputs{
                    sig.returns.push(AbiParam::new(from_base_type(*output)));
                }
                
                let id = self.module.declare_function(&function.name, Linkage::Export, &sig).unwrap();

                let decl = entry.insert(FunctionDeclaration {
                    defined: false,
                    id,
                    signature: sig
                });

                decl
            },
        }
    }

    pub fn get_function(&self, module: ModuleRef, function: FunctionRef) -> *const u8{
        let decl = &self.functions[&(module, function)];
        if !decl.defined{
            panic!("Undefined method");
        }

        self.module.get_finalized_function(decl.id)
    }
}

struct FunctionDeclaration{
    defined: bool,
    id: FuncId,
    signature: Signature,
}

fn from_base_type(type_: corrosion_base::Type) -> Type{
        match type_ {
            corrosion_base::Type::I8 => types::I8,
            corrosion_base::Type::I16 => types::I16,
            corrosion_base::Type::I32 => types::I32,
            corrosion_base::Type::I64 => types::I64,
            corrosion_base::Type::F32 => types::F32,
            corrosion_base::Type::F64 => types::F64,
            corrosion_base::Type::Ptr => types::R64,
        }
}