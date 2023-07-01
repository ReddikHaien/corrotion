use std::fs;

use corrosion_base::{ModuleBuilder, Type};

fn main() {
    let mut mb = ModuleBuilder::new();
    let f_a = mb.new_function();
    let mut fb = mb.funtion_builder(f_a);

    fb.add_input(Type::F32);
    fb.add_input(Type::F32);
    fb.add_output(Type::F32);
    let block = fb.create_block();

    let mut bb = fb.block_builder(block);
    bb.into_entry_block();

    let [a, b] = *bb.get_params() else {unreachable!("Entry block should have 2 values")};
    
    let out = bb.add_values(a, b);
    bb.return_(&[out]);

    let module = mb.build();


    println!("{:#?}",module);    
}
