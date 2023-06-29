use std::fs;

pub mod frontend;
pub mod parser;

fn main() {
    let src = fs::read_to_string("./test.txt");

    let src = match src {
        Ok(s) => s,
        Err(e) => panic!("{}",e),
    };

    let module = match frontend::file_parser::module(&src){
        Ok(module) => module,
        Err(e) => panic!("{}",e),
    };

    println!("{:#?}", module)

}
