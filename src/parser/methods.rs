use cranelift::prelude::{FunctionBuilder, InstBuilder};

use super::{ResolvedTypes, CodeGenerator, ValueType};

pub trait MethodResolver{
    fn correct_args(&self, values: &[ResolvedTypes]) -> bool;
    fn create_ir(&self, values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>;
}

#[macro_export]
macro_rules! impl_resolver {
    (
        $(($name:ident, ( $($arg:ident),* ), $resolver:expr)),*
    ) => {
        $(
            pub struct $name;
            impl MethodResolver for $name{
                fn correct_args(&self, values: &[ResolvedTypes]) -> bool{
                    let mut i = 0;
                    true $(
                        || match &values[{i+=1; i-1}] {$crate::parser::ResolvedTypes::$arg{..} => true, _ => false}
                    )*
                }

                fn create_ir(&self, values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>
                {
                    let resolver = $resolver;
                    (resolver)(values, builder)
                }
            }
        )*

        pub fn register_builtin_methods(map: &mut std::collections::hash_map::HashMap<&'static str, Box<dyn MethodResolver>>)
        {
            $(
                map.insert(stringify!($name), Box::new($name));
            )*
        }
    };
}


impl_resolver!(
    (add, (Value, Value), add_values),
    (sub, (Value, Value), sub_values),
    (mul, (Value, Value), mul_values),
    (r#return, (), return_values)
);


fn return_values(values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>{

    let values = values.iter().map(|x| x.as_value().0).collect::<Vec<_>>();
    builder.ins().return_(&values);
    Vec::with_capacity(0)
}

fn add_values(values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>{
    let (a, a_type) = values[0].as_value();
    let (b, b_type) = values[1].as_value();

    let (result, result_type) = match (a_type, b_type){
        (ValueType::Integer, ValueType::Integer) => (builder.ins().iadd(a, b), ValueType::Integer),
        (ValueType::F32, ValueType::F32) => (builder.ins().fadd(a, b), ValueType::F32),
        (ValueType::F64, ValueType::F64) => (builder.ins().fadd(a, b), ValueType::F64),
        (x, y) => panic!("Can't add together values of types {:?} and {:?}", x, y)
    };

    vec![ResolvedTypes::Value(result, result_type)]
}

fn sub_values(values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>{
    let (a, a_type) = values[0].as_value();
    let (b, b_type) = values[1].as_value();

    let (result, result_type) = match (a_type, b_type){
        (ValueType::Integer, ValueType::Integer) => (builder.ins().isub(a, b), ValueType::Integer),
        (ValueType::F32, ValueType::F32) => (builder.ins().fsub(a, b), ValueType::F32),
        (ValueType::F64, ValueType::F64) => (builder.ins().fsub(a, b), ValueType::F64),
        (x, y) => panic!("Can't add together values of types {:?} and {:?}", x, y)
    };

    vec![ResolvedTypes::Value(result, result_type)]
}

fn mul_values(values: &[ResolvedTypes], builder: &mut FunctionBuilder) -> Vec<ResolvedTypes>{
    let (a, a_type) = values[0].as_value();
    let (b, b_type) = values[1].as_value();

    let (result, result_type) = match (a_type, b_type){
        (ValueType::Integer, ValueType::Integer) => (builder.ins().imul(a, b), ValueType::Integer),
        (ValueType::F32, ValueType::F32) => (builder.ins().fmul(a, b), ValueType::F32),
        (ValueType::F64, ValueType::F64) => (builder.ins().fmul(a, b), ValueType::F64),
        (x, y) => panic!("Can't add together values of types {:?} and {:?}", x, y)
    };

    vec![ResolvedTypes::Value(result, result_type)]
}
