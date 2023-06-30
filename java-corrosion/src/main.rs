use std::{fs::File, io::Read};

use zip::ZipArchive;

pub mod frontend;

fn main() {
    let mut archive = ZipArchive::new(File::open("./jagexappletviewer.jar").unwrap()).unwrap();

    for file in 0..archive.len(){
        let file = archive.by_index(file).unwrap();
        if file.name().ends_with(".class"){
            println!("{}", file.name())
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer);
        }
    }
    
}
